//! Connect usage query operations.

use std::sync::Arc;

use async_trait::async_trait;
use pipedream_connect_types::{ConnectUsageResponse, ListUsageRequest};

use crate::client::RequestDispatcher;
use crate::error::Result;

use super::admin_operations;
use super::operation;

/// Usage operations, consumed through a shared trait object.
#[cfg_attr(
    any(test, feature = "unimock"),
    unimock::unimock(api = UsageApiMock)
)]
#[async_trait]
pub trait UsageApi: Send + Sync {
    /// Lists usage records for an inclusive epoch-second time window.
    async fn list(&self, request: ListUsageRequest) -> Result<ConnectUsageResponse>;
}

/// A usage handle backed by the shared dispatcher.
#[derive(Clone)]
pub struct Usage {
    dispatcher: Arc<dyn RequestDispatcher + Send + Sync>,
}

impl Usage {
    /// Injects shared dispatch without sending a request.
    pub fn new(dispatcher: Arc<dyn RequestDispatcher + Send + Sync>) -> Self {
        Self { dispatcher }
    }
}

#[async_trait]
impl UsageApi for Usage {
    async fn list(&self, request: ListUsageRequest) -> Result<ConnectUsageResponse> {
        operation::json(&self.dispatcher, admin_operations::list_usage(&request)).await
    }
}

#[cfg(test)]
#[path = "_tests_/usage_tests.rs"]
mod usage_tests;
