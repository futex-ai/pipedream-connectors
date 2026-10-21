//! Workflow invocation for one external user.

use std::sync::Arc;

use async_trait::async_trait;
use http::Method;
use pipedream_connect_types::ExternalUserId;

use crate::client::RequestDispatcher;
use crate::error::{Error, InvalidRequest, Result};
use crate::transport::HttpResponse;

use super::admin_operations;
use super::operation;

/// Workflow invocation operations, consumed through a shared trait object.
#[cfg_attr(
    any(test, feature = "unimock"),
    unimock::unimock(api = WorkflowsApiMock)
)]
#[async_trait]
pub trait WorkflowsApi: Send + Sync {
    /// Invokes an HTTPS workflow endpoint for one external user.
    async fn invoke_for_external_user(
        &self,
        endpoint: String,
        external_user_id: ExternalUserId,
        method: Method,
        body: Vec<u8>,
    ) -> Result<HttpResponse>;
}

/// A workflow invocation handle backed by the shared dispatcher.
#[derive(Clone)]
pub struct Workflows {
    dispatcher: Arc<dyn RequestDispatcher + Send + Sync>,
}

impl Workflows {
    /// Injects shared dispatch without sending a request.
    pub fn new(dispatcher: Arc<dyn RequestDispatcher + Send + Sync>) -> Self {
        Self { dispatcher }
    }
}

#[async_trait]
impl WorkflowsApi for Workflows {
    async fn invoke_for_external_user(
        &self,
        endpoint: String,
        external_user_id: ExternalUserId,
        method: Method,
        body: Vec<u8>,
    ) -> Result<HttpResponse> {
        if !matches!(
            method,
            Method::GET | Method::POST | Method::PUT | Method::DELETE | Method::PATCH
        ) {
            return Err(Error::InvalidRequest(InvalidRequest::WorkflowMethod));
        }
        operation::dispatch(
            &self.dispatcher,
            admin_operations::invoke_workflow(&endpoint, method, &external_user_id, body),
        )
        .await
    }
}

#[cfg(test)]
#[path = "_tests_/workflows_tests.rs"]
mod workflows_tests;
