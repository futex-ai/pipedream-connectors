//! Deployed trigger lifecycle operations and tenant-scoped destinations.

use std::sync::Arc;

use async_trait::async_trait;
use pipedream_connect_types::{
    DeleteTriggerRequest, DeployedTriggerId, DeployedTriggerQuery, GetTriggerEventsResponse,
    GetTriggerResponse, GetTriggerWebhooksResponse, GetTriggerWorkflowsResponse,
    GetTriggersResponse, GetWebhookWithSigningKeyResponse, ListDeployedTriggersRequest,
    ListTriggerEventsRequest, UpdateTriggerRequest, UpdateTriggerWebhooksRequest,
    UpdateTriggerWorkflowsRequest, WebhookId,
};

use crate::client::RequestDispatcher;
use crate::error::{Error, Result, WireFailure};

use super::{operation, trigger_operations};

/// Deployed trigger lifecycle operations, consumed through a shared trait object.
#[cfg_attr(
    any(test, feature = "unimock"),
    unimock::unimock(api = DeployedTriggersApiMock)
)]
#[async_trait]
pub trait DeployedTriggersApi: Send + Sync {
    /// Lists deployed triggers for one external user.
    async fn list(&self, request: ListDeployedTriggersRequest) -> Result<GetTriggersResponse>;
    /// Retrieves one deployed trigger and checks its identity when available.
    async fn get(
        &self,
        trigger_id: DeployedTriggerId,
        request: DeployedTriggerQuery,
    ) -> Result<GetTriggerResponse>;
    /// Updates one deployed trigger and checks its identity when available.
    async fn update(
        &self,
        trigger_id: DeployedTriggerId,
        query: DeployedTriggerQuery,
        request: UpdateTriggerRequest,
    ) -> Result<GetTriggerResponse>;
    /// Deletes one deployed trigger, with optional missing-resource tolerance.
    async fn delete(
        &self,
        trigger_id: DeployedTriggerId,
        request: DeleteTriggerRequest,
    ) -> Result<()>;
    /// Lists recent events for one deployed trigger.
    async fn list_events(
        &self,
        trigger_id: DeployedTriggerId,
        request: ListTriggerEventsRequest,
    ) -> Result<GetTriggerEventsResponse>;
    /// Gets workflow endpoints for one deployed trigger.
    async fn get_workflows(
        &self,
        trigger_id: DeployedTriggerId,
        request: DeployedTriggerQuery,
    ) -> Result<GetTriggerWorkflowsResponse>;
    /// Replaces workflow endpoints for one deployed trigger.
    async fn set_workflows(
        &self,
        trigger_id: DeployedTriggerId,
        query: DeployedTriggerQuery,
        request: UpdateTriggerWorkflowsRequest,
    ) -> Result<GetTriggerWorkflowsResponse>;
    /// Lists webhook destinations for one deployed trigger.
    async fn list_webhooks(
        &self,
        trigger_id: DeployedTriggerId,
        request: DeployedTriggerQuery,
    ) -> Result<GetTriggerWebhooksResponse>;
    /// Replaces webhook destinations for one deployed trigger.
    async fn set_webhooks(
        &self,
        trigger_id: DeployedTriggerId,
        query: DeployedTriggerQuery,
        request: UpdateTriggerWebhooksRequest,
    ) -> Result<GetTriggerWebhooksResponse>;
    /// Retrieves one webhook and its signing key.
    async fn get_webhook(
        &self,
        trigger_id: DeployedTriggerId,
        webhook_id: WebhookId,
        request: DeployedTriggerQuery,
    ) -> Result<GetWebhookWithSigningKeyResponse>;
    /// Regenerates one webhook signing key.
    async fn regenerate_webhook_signing_key(
        &self,
        trigger_id: DeployedTriggerId,
        webhook_id: WebhookId,
        request: DeployedTriggerQuery,
    ) -> Result<GetWebhookWithSigningKeyResponse>;
}

/// A deployed-trigger handle backed by the shared authenticated dispatcher.
#[derive(Clone)]
pub struct DeployedTriggers {
    dispatcher: Arc<dyn RequestDispatcher + Send + Sync>,
}

impl DeployedTriggers {
    /// Injects shared dispatch without sending a request.
    pub fn new(dispatcher: Arc<dyn RequestDispatcher + Send + Sync>) -> Self {
        Self { dispatcher }
    }
}

#[async_trait]
impl DeployedTriggersApi for DeployedTriggers {
    async fn list(&self, request: ListDeployedTriggersRequest) -> Result<GetTriggersResponse> {
        operation::json(&self.dispatcher, trigger_operations::list(&request)).await
    }

    async fn get(
        &self,
        trigger_id: DeployedTriggerId,
        request: DeployedTriggerQuery,
    ) -> Result<GetTriggerResponse> {
        let response: GetTriggerResponse = operation::json(
            &self.dispatcher,
            trigger_operations::get(&trigger_id, &request),
        )
        .await?;
        trigger_operations::check_trigger(response, &trigger_id)
    }

    async fn update(
        &self,
        trigger_id: DeployedTriggerId,
        query: DeployedTriggerQuery,
        request: UpdateTriggerRequest,
    ) -> Result<GetTriggerResponse> {
        let response: GetTriggerResponse = operation::json(
            &self.dispatcher,
            trigger_operations::update(&trigger_id, &query, &request),
        )
        .await?;
        trigger_operations::check_trigger(response, &trigger_id)
    }

    async fn delete(
        &self,
        trigger_id: DeployedTriggerId,
        request: DeleteTriggerRequest,
    ) -> Result<()> {
        self.dispatcher
            .dispatch(trigger_operations::delete(&trigger_id, &request)?)
            .await
            .map(|_| ())
    }

    async fn list_events(
        &self,
        trigger_id: DeployedTriggerId,
        request: ListTriggerEventsRequest,
    ) -> Result<GetTriggerEventsResponse> {
        let response: GetTriggerEventsResponse = operation::json(
            &self.dispatcher,
            trigger_operations::events(&trigger_id, &request),
        )
        .await?;
        if response.data.len() > usize::from(request.limit) {
            return Err(Error::internal(WireFailure::Pagination));
        }
        Ok(response)
    }

    async fn get_workflows(
        &self,
        trigger_id: DeployedTriggerId,
        request: DeployedTriggerQuery,
    ) -> Result<GetTriggerWorkflowsResponse> {
        operation::json(
            &self.dispatcher,
            trigger_operations::workflows(&trigger_id, &request),
        )
        .await
    }

    async fn set_workflows(
        &self,
        trigger_id: DeployedTriggerId,
        query: DeployedTriggerQuery,
        request: UpdateTriggerWorkflowsRequest,
    ) -> Result<GetTriggerWorkflowsResponse> {
        operation::json(
            &self.dispatcher,
            trigger_operations::update_workflows(&trigger_id, &query, &request),
        )
        .await
    }

    async fn list_webhooks(
        &self,
        trigger_id: DeployedTriggerId,
        request: DeployedTriggerQuery,
    ) -> Result<GetTriggerWebhooksResponse> {
        operation::json(
            &self.dispatcher,
            trigger_operations::webhooks(&trigger_id, &request),
        )
        .await
    }

    async fn set_webhooks(
        &self,
        trigger_id: DeployedTriggerId,
        query: DeployedTriggerQuery,
        request: UpdateTriggerWebhooksRequest,
    ) -> Result<GetTriggerWebhooksResponse> {
        operation::json(
            &self.dispatcher,
            trigger_operations::update_webhooks(&trigger_id, &query, &request),
        )
        .await
    }

    async fn get_webhook(
        &self,
        trigger_id: DeployedTriggerId,
        webhook_id: WebhookId,
        request: DeployedTriggerQuery,
    ) -> Result<GetWebhookWithSigningKeyResponse> {
        let response = operation::json(
            &self.dispatcher,
            trigger_operations::webhook(&trigger_id, &webhook_id, &request),
        )
        .await?;
        trigger_operations::check_webhook(response, &webhook_id)
    }

    async fn regenerate_webhook_signing_key(
        &self,
        trigger_id: DeployedTriggerId,
        webhook_id: WebhookId,
        request: DeployedTriggerQuery,
    ) -> Result<GetWebhookWithSigningKeyResponse> {
        let response = operation::json(
            &self.dispatcher,
            trigger_operations::regenerate_webhook_signing_key(&trigger_id, &webhook_id, &request),
        )
        .await?;
        trigger_operations::check_webhook(response, &webhook_id)
    }
}

#[cfg(test)]
#[path = "_tests_/deployed_triggers/mod.rs"]
mod deployed_triggers_tests;
