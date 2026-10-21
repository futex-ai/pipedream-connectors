//! Trigger deployment models and requests.

use serde::{Deserialize, Serialize};

use crate::WorkflowEndpoint;

/// Trigger webhook replacement request.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UpdateTriggerWebhooksRequest {
    /// Destination URLs.
    pub webhook_urls: Vec<String>,
}

/// Trigger workflow replacement request.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UpdateTriggerWorkflowsRequest {
    /// Workflow endpoint values.
    pub workflow_ids: Vec<WorkflowEndpoint>,
}
