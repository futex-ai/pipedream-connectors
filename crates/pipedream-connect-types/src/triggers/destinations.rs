//! Trigger deployment models and requests.

use serde::{Deserialize, Serialize};

use crate::{SecretString, UnknownFields, WebhookId, WorkflowEndpoint};

/// One deployed-trigger webhook destination.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TriggerWebhook {
    /// Webhook identifier.
    pub id: WebhookId,
    /// Destination URL.
    pub url: String,
    /// Signing key when returned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signing_key: Option<SecretString>,
    /// Whether a signing key is configured.
    pub signing_key_set: bool,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Webhook list or replacement response.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TriggerWebhooks {
    /// Effective destination URLs.
    pub webhook_urls: Vec<String>,
    /// Destination metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webhooks: Option<Vec<TriggerWebhook>>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Workflow endpoints associated with a deployed trigger.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TriggerWorkflows {
    /// Workflow endpoint values.
    pub workflow_ids: Vec<WorkflowEndpoint>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}
