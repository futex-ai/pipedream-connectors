//! Optional reqwest transport with no redirects, retries, or unbounded bodies.

use async_trait::async_trait;

use crate::config::HttpOptions;
use crate::error::{Error, Result, WireFailure};
use crate::transport::{HttpRequest, HttpResponse, HttpTransport};

/// Rustls HTTP transport configured at the composition root.
#[derive(Clone, Debug)]
pub struct ReqwestTransport {
    client: reqwest::Client,
}

impl ReqwestTransport {
    /// Configures connection and total timeouts and disables redirects and retries.
    pub fn new(options: &HttpOptions) -> Result<Self> {
        let client = match reqwest::Client::builder()
            .tls_backend_rustls()
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .connect_timeout(options.connect_timeout)
            .timeout(options.request_timeout)
            .build()
        {
            Ok(client) => client,
            Err(_) => return Err(Error::internal(WireFailure::ClientConstruction)),
        };
        Ok(Self { client })
    }
}

#[async_trait]
impl HttpTransport for ReqwestTransport {
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse> {
        let limit = request.response_limit;
        let mut builder = self
            .client
            .request(request.method, request.url.expose_secret())
            .headers(request.headers)
            .timeout(request.timeout);
        if let Some(body) = request.body {
            builder = builder.body(body);
        }
        let mut response = match builder.send().await {
            Ok(response) => response,
            Err(_) => return Err(Error::Unavailable { status: None }),
        };
        if response
            .content_length()
            .is_some_and(|length| length > limit as u64)
        {
            return Err(Error::ResponseTooLarge { limit });
        }
        let status = response.status();
        let headers = response.headers().clone();
        let mut body = Vec::new();
        loop {
            match response.chunk().await {
                Ok(Some(chunk)) => {
                    if chunk.len() > limit.saturating_sub(body.len()) {
                        return Err(Error::ResponseTooLarge { limit });
                    }
                    body.extend_from_slice(&chunk);
                }
                Ok(None) => {
                    return Ok(HttpResponse {
                        status,
                        headers,
                        body,
                    });
                }
                Err(_) => return Err(Error::Unavailable { status: None }),
            }
        }
    }
}
