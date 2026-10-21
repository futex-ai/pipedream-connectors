//! Trigger deployment models and requests.

use serde::{Deserialize, Serialize};

use crate::{Result, WorkflowEndpoint};

use super::requests::validate_trigger_webhook_url;

/// Trigger webhook replacement request.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UpdateTriggerWebhooksRequest {
    /// Destination URLs.
    pub webhook_urls: Vec<String>,
}

impl UpdateTriggerWebhooksRequest {
    /// Validates every replacement callback URL.
    pub fn validate(&self) -> Result<()> {
        for url in &self.webhook_urls {
            validate_trigger_webhook_url(url)?;
        }
        Ok(())
    }
}

/// Trigger workflow replacement request.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UpdateTriggerWorkflowsRequest {
    /// Workflow endpoint values.
    pub workflow_ids: Vec<WorkflowEndpoint>,
}
