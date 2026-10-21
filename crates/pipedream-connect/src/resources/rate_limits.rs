//! Custom rate-limit token operations.

use std::sync::Arc;

use async_trait::async_trait;
use pipedream_connect_types::{RateLimitRequest, RateLimitToken};

use crate::client::RequestDispatcher;
use crate::error::Result;

use super::admin_operations;
use super::operation;

/// Custom rate-limit operations, consumed through a shared trait object.
#[cfg_attr(
    any(test, feature = "unimock"),
    unimock::unimock(api = RateLimitsApiMock)
)]
#[async_trait]
pub trait RateLimitsApi: Send + Sync {
    /// Creates a custom rate-limit token for later client configuration.
    async fn create(&self, request: RateLimitRequest) -> Result<RateLimitToken>;
}

/// A rate-limit handle backed by the shared dispatcher.
#[derive(Clone)]
pub struct RateLimits {
    dispatcher: Arc<dyn RequestDispatcher + Send + Sync>,
}

impl RateLimits {
    /// Injects shared dispatch without sending a request.
    pub fn new(dispatcher: Arc<dyn RequestDispatcher + Send + Sync>) -> Self {
        Self { dispatcher }
    }
}

#[async_trait]
impl RateLimitsApi for RateLimits {
    async fn create(&self, request: RateLimitRequest) -> Result<RateLimitToken> {
        operation::json(
            &self.dispatcher,
            admin_operations::create_rate_limit(&request),
        )
        .await
    }
}

#[cfg(test)]
#[path = "_tests_/rate_limits_tests.rs"]
mod rate_limits_tests;
