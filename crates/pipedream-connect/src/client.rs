//! Client composition and the shared authenticated request dispatcher.

use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use http::Method;
use pipedream_connect_types::SecretString;

use crate::config::{Authentication, ClientConfig};
use crate::error::Result;
use crate::failure::{check_limit, ensure_status, retry_after};
use crate::headers::authenticated;
use crate::request::{ApiRequest, ResponseMode, build_url};
use crate::runtime::{Clock, Sleeper};
use crate::token::{AccessTokenProvider, ClientCredentialsProvider, StaticTokenProvider};
use crate::transport::{HttpRequest, HttpResponse, HttpTransport};

/// Runtime boundary shared by every resource operation.
#[cfg_attr(any(test, feature = "unimock"), unimock::unimock(api = RequestDispatcherMock))]
#[async_trait]
pub trait RequestDispatcher: Send + Sync {
    /// Authenticates, sends, bounds, and maps an operation without mutation retries.
    async fn dispatch(&self, request: ApiRequest) -> Result<HttpResponse>;
}

/// Composition root for Connect resources and their shared runtime boundaries.
#[derive(Clone)]
pub struct PipedreamClient {
    config: ClientConfig,
    transport: Arc<dyn HttpTransport + Send + Sync>,
    clock: Arc<dyn Clock + Send + Sync>,
    sleeper: Arc<dyn Sleeper + Send + Sync>,
    tokens: Arc<dyn AccessTokenProvider + Send + Sync>,
}

impl PipedreamClient {
    /// Injects transport, clock, and sleeper. Construction sends no request.
    pub fn new(
        config: ClientConfig,
        transport: Arc<dyn HttpTransport + Send + Sync>,
        clock: Arc<dyn Clock + Send + Sync>,
        sleeper: Arc<dyn Sleeper + Send + Sync>,
    ) -> Result<Self> {
        let tokens: Arc<dyn AccessTokenProvider + Send + Sync> = match &config.authentication {
            Authentication::ClientCredentials(_) => Arc::new(ClientCredentialsProvider::new(
                config.clone(),
                transport.clone(),
                clock.clone(),
            )?),
            Authentication::StaticBearer(token) => {
                Arc::new(StaticTokenProvider::new(token.clone())?)
            }
        };
        Ok(Self {
            config,
            transport,
            clock,
            sleeper,
            tokens,
        })
    }

    /// Injects the sleeper for an explicitly configured GET retry policy.
    pub fn with_sleeper(mut self, sleeper: Arc<dyn Sleeper + Send + Sync>) -> Self {
        self.sleeper = sleeper;
        self
    }

    /// Selects a host-managed access-token provider.
    pub fn with_token_provider(
        mut self,
        provider: Arc<dyn AccessTokenProvider + Send + Sync>,
    ) -> Self {
        self.tokens = provider;
        self
    }

    /// Returns a shared dispatcher. Clones share the OAuth cache.
    pub fn dispatcher(&self) -> Arc<dyn RequestDispatcher + Send + Sync> {
        Arc::new(self.clone())
    }

    /// Returns the immutable configuration.
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }
}

#[async_trait]
impl RequestDispatcher for PipedreamClient {
    async fn dispatch(&self, request: ApiRequest) -> Result<HttpResponse> {
        let url = build_url(&self.config, &request.path, &request.query)?;
        let mut attempts = 0;
        loop {
            let token = self.tokens.access_token().await?;
            let headers = authenticated(
                &self.config,
                &token,
                request.headers.clone(),
                matches!(
                    request.mode,
                    ResponseMode::Json | ResponseMode::Execution { .. }
                ),
                request.json_body,
            )?;
            let response = self
                .transport
                .send(HttpRequest {
                    method: request.method.clone(),
                    url: SecretString::new(url.as_str()),
                    headers,
                    body: request.body.clone(),
                    response_limit: self.config.http.response_limit,
                    timeout: self.config.http.request_timeout,
                })
                .await?;
            attempts += 1;
            if response.status == 401 {
                self.tokens.invalidate(&token).await;
            }
            check_limit(&response, self.config.http.response_limit)?;
            if request.method == Method::GET
                && request.mode != ResponseMode::PassThrough
                && let Some(policy) = &self.config.retry
                && policy.retries(response.status.as_u16(), attempts)
                && let Some(delay) = policy.delay(retry_after(&response.headers, self.clock.now()))
            {
                tracing::debug!(
                    attempts,
                    status = response.status.as_u16(),
                    "retrying Pipedream GET"
                );
                self.sleeper.sleep(delay).await;
                continue;
            }
            ensure_status(&response, request.mode, self.clock.now())?;
            return Ok(response);
        }
    }
}

impl fmt::Debug for PipedreamClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PipedreamClient")
            .field("config", &self.config)
            .field("tokens", &"[REDACTED]")
            .finish_non_exhaustive()
    }
}
