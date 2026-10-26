//! Single-request HTTP boundary with redacted diagnostics and explicit bounds.

use std::fmt;
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use http::{HeaderMap, Method, StatusCode};
use pipedream_connect_types::SecretString;
use serde::de::DeserializeOwned;

use crate::error::Result;
use crate::json::decode;

/// One HTTP request. A transport must never redirect or retry it.
#[derive(Clone)]
pub struct HttpRequest {
    /// HTTP operation.
    pub method: Method,
    /// Absolute URL. Explicit access is required because queries may hold secrets.
    pub url: SecretString,
    /// Exact headers, which can contain credentials.
    pub headers: HeaderMap,
    /// Serialized request body shared by all dispatch attempts.
    pub body: Option<Bytes>,
    /// Maximum response body bytes, enforced before and during reading.
    pub response_limit: usize,
    /// Total request timeout, including the response body.
    pub timeout: Duration,
}

impl fmt::Debug for HttpRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HttpRequest")
            .field("method", &self.method)
            .field("url", &"[REDACTED]")
            .field("headers", &"[REDACTED]")
            .field("body_bytes", &self.body.as_ref().map(Bytes::len))
            .field("response_limit", &self.response_limit)
            .field("timeout", &self.timeout)
            .finish()
    }
}

/// A complete response whose body has passed the byte limit.
#[derive(Clone)]
pub struct HttpResponse {
    /// HTTP response status.
    pub status: StatusCode,
    /// Every response header, including repeated and non-UTF-8 values.
    pub headers: HeaderMap,
    /// Exact response body bytes.
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Decodes JSON without retaining data-bearing decoder error messages.
    #[track_caller]
    pub fn json<T: DeserializeOwned>(&self) -> Result<T> {
        decode(&self.body)
    }
}

impl fmt::Debug for HttpResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HttpResponse")
            .field("status", &self.status)
            .field("headers", &"[REDACTED]")
            .field("body_bytes", &self.body.len())
            .finish()
    }
}

/// Shared transport consumed through `Arc<dyn HttpTransport + Send + Sync>`.
#[cfg_attr(any(test, feature = "unimock"), unimock::unimock(api = HttpTransportMock))]
#[async_trait]
pub trait HttpTransport: Send + Sync {
    /// Sends once, without redirects, within the request timeout and byte cap.
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse>;
}
