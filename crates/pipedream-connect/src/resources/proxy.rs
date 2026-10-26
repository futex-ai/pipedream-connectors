//! Authenticated proxy requests with strict caller-header admission.

use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use http::{HeaderMap, HeaderName, HeaderValue, Method};
use pipedream_connect_types::AccountRef;

use crate::client::RequestDispatcher;
use crate::error::Result;
use crate::transport::HttpResponse;

use super::auth_operations;

/// Supported authenticated proxy methods.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProxyMethod {
    /// Retrieves an upstream resource.
    Get,
    /// Creates or invokes an upstream resource.
    Post,
    /// Replaces an upstream resource.
    Put,
    /// Partially updates an upstream resource.
    Patch,
    /// Deletes an upstream resource.
    Delete,
}

impl From<ProxyMethod> for Method {
    fn from(method: ProxyMethod) -> Self {
        match method {
            ProxyMethod::Get => Self::GET,
            ProxyMethod::Post => Self::POST,
            ProxyMethod::Put => Self::PUT,
            ProxyMethod::Patch => Self::PATCH,
            ProxyMethod::Delete => Self::DELETE,
        }
    }
}

/// One authenticated request to a Connect proxy target.
#[derive(Clone)]
pub struct ProxyRequest {
    /// Account and external-user identity for the upstream call.
    pub account: AccountRef,
    /// HTTP method sent to Connect.
    pub method: ProxyMethod,
    /// Exact absolute HTTPS target or relative path.
    pub target: String,
    /// Unprefixed upstream headers supplied by the caller.
    pub headers: HeaderMap,
    /// Exact bytes sent to the upstream target.
    pub body: Option<Vec<u8>>,
}

impl ProxyRequest {
    /// Creates a request for a selected method.
    pub fn new(
        account: impl Into<AccountRef>,
        method: ProxyMethod,
        target: impl Into<String>,
    ) -> Self {
        Self {
            account: account.into(),
            method,
            target: target.into(),
            headers: HeaderMap::new(),
            body: None,
        }
    }

    /// Creates a GET proxy request.
    pub fn get(account: impl Into<AccountRef>, target: impl Into<String>) -> Self {
        Self::new(account, ProxyMethod::Get, target)
    }

    /// Creates a POST proxy request.
    pub fn post(account: impl Into<AccountRef>, target: impl Into<String>) -> Self {
        Self::new(account, ProxyMethod::Post, target)
    }

    /// Creates a PUT proxy request.
    pub fn put(account: impl Into<AccountRef>, target: impl Into<String>) -> Self {
        Self::new(account, ProxyMethod::Put, target)
    }

    /// Creates a PATCH proxy request.
    pub fn patch(account: impl Into<AccountRef>, target: impl Into<String>) -> Self {
        Self::new(account, ProxyMethod::Patch, target)
    }

    /// Creates a DELETE proxy request.
    pub fn delete(account: impl Into<AccountRef>, target: impl Into<String>) -> Self {
        Self::new(account, ProxyMethod::Delete, target)
    }

    /// Merges caller headers without sending a request.
    pub fn with_headers(mut self, headers: HeaderMap) -> Self {
        self.headers.extend(headers);
        self
    }

    /// Appends one caller header without sending a request.
    pub fn with_header(mut self, name: HeaderName, value: HeaderValue) -> Self {
        self.headers.append(name, value);
        self
    }

    /// Replaces the exact byte body without changing its encoding.
    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = Some(body.into());
        self
    }
}

impl fmt::Debug for ProxyRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProxyRequest")
            .field("account", &self.account)
            .field("method", &self.method)
            .field("target", &"[REDACTED]")
            .field("headers", &"[REDACTED]")
            .field("body_bytes", &self.body.as_ref().map(Vec::len))
            .finish()
    }
}

/// A complete proxy response with every returned header and body byte.
pub type ProxyResponse = HttpResponse;

/// Authenticated proxy operations, consumed through a shared trait object.
#[cfg_attr(
    any(test, feature = "unimock"),
    unimock::unimock(api = ProxyApiMock)
)]
#[async_trait]
pub trait ProxyApi: Send + Sync {
    /// Sends a proxy request using its selected method.
    async fn send(&self, request: ProxyRequest) -> Result<ProxyResponse>;
    /// Sends a GET proxy request.
    async fn get(&self, request: ProxyRequest) -> Result<ProxyResponse>;
    /// Sends a POST proxy request.
    async fn post(&self, request: ProxyRequest) -> Result<ProxyResponse>;
    /// Sends a PUT proxy request.
    async fn put(&self, request: ProxyRequest) -> Result<ProxyResponse>;
    /// Sends a PATCH proxy request.
    async fn patch(&self, request: ProxyRequest) -> Result<ProxyResponse>;
    /// Sends a DELETE proxy request.
    async fn delete(&self, request: ProxyRequest) -> Result<ProxyResponse>;
}

/// A proxy handle backed by the shared authenticated dispatcher.
#[derive(Clone)]
pub struct Proxy {
    dispatcher: Arc<dyn RequestDispatcher + Send + Sync>,
}

impl Proxy {
    /// Injects shared dispatch without sending a request.
    pub fn new(dispatcher: Arc<dyn RequestDispatcher + Send + Sync>) -> Self {
        Self { dispatcher }
    }
}

#[cfg(test)]
#[path = "_tests_/proxy_tests.rs"]
mod proxy_tests;

#[async_trait]
impl ProxyApi for Proxy {
    async fn send(&self, request: ProxyRequest) -> Result<ProxyResponse> {
        self.send_request(request).await
    }

    async fn get(&self, request: ProxyRequest) -> Result<ProxyResponse> {
        let mut request = request;
        request.method = ProxyMethod::Get;
        self.send_request(request).await
    }

    async fn post(&self, request: ProxyRequest) -> Result<ProxyResponse> {
        let mut request = request;
        request.method = ProxyMethod::Post;
        self.send_request(request).await
    }

    async fn put(&self, request: ProxyRequest) -> Result<ProxyResponse> {
        let mut request = request;
        request.method = ProxyMethod::Put;
        self.send_request(request).await
    }

    async fn patch(&self, request: ProxyRequest) -> Result<ProxyResponse> {
        let mut request = request;
        request.method = ProxyMethod::Patch;
        self.send_request(request).await
    }

    async fn delete(&self, request: ProxyRequest) -> Result<ProxyResponse> {
        let mut request = request;
        request.method = ProxyMethod::Delete;
        self.send_request(request).await
    }
}

impl Proxy {
    async fn send_request(&self, request: ProxyRequest) -> Result<ProxyResponse> {
        self.dispatcher
            .dispatch(auth_operations::proxy(request)?)
            .await
    }
}
