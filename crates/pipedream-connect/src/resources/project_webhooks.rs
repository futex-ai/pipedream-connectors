//! Project environment webhook administration operations.

use std::sync::Arc;

use async_trait::async_trait;
use pipedream_connect_types::{
    GetWebhookResponse, GetWebhookWithSigningKeyResponse, SetWebhookRequest, SetWebhookResponse,
};

use crate::client::RequestDispatcher;
use crate::error::Result;

use super::admin_operations;
use super::operation;

/// Project environment webhook operations, consumed through a shared trait object.
#[cfg_attr(
    any(test, feature = "unimock"),
    unimock::unimock(api = ProjectWebhooksApiMock)
)]
#[async_trait]
pub trait ProjectWebhooksApi: Send + Sync {
    /// Retrieves the configured webhook, including an absent webhook.
    async fn get(&self) -> Result<GetWebhookResponse>;
    /// Sets the webhook destination.
    async fn set(&self, request: SetWebhookRequest) -> Result<SetWebhookResponse>;
    /// Deletes the configured webhook.
    async fn delete(&self) -> Result<()>;
    /// Regenerates and returns the webhook signing key.
    async fn regenerate_signing_key(&self) -> Result<GetWebhookWithSigningKeyResponse>;
}

/// A project environment webhook handle backed by the shared dispatcher.
#[derive(Clone)]
pub struct ProjectWebhooks {
    dispatcher: Arc<dyn RequestDispatcher + Send + Sync>,
}

impl ProjectWebhooks {
    /// Injects shared dispatch without sending a request.
    pub fn new(dispatcher: Arc<dyn RequestDispatcher + Send + Sync>) -> Self {
        Self { dispatcher }
    }
}

#[async_trait]
impl ProjectWebhooksApi for ProjectWebhooks {
    async fn get(&self) -> Result<GetWebhookResponse> {
        operation::json(
            &self.dispatcher,
            Ok(admin_operations::get_project_webhook()),
        )
        .await
    }

    async fn set(&self, request: SetWebhookRequest) -> Result<SetWebhookResponse> {
        operation::json(
            &self.dispatcher,
            admin_operations::set_project_webhook(&request),
        )
        .await
    }

    async fn delete(&self) -> Result<()> {
        operation::no_content(
            &self.dispatcher,
            Ok(admin_operations::delete_project_webhook()),
        )
        .await
    }

    async fn regenerate_signing_key(&self) -> Result<GetWebhookWithSigningKeyResponse> {
        operation::json(
            &self.dispatcher,
            Ok(admin_operations::regenerate_project_webhook()),
        )
        .await
    }
}

#[cfg(test)]
#[path = "_tests_/project_webhooks_tests.rs"]
mod project_webhooks_tests;
