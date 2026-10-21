//! Trigger deployment, webhook, workflow, and event models.

mod emitters;
pub use emitters::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    ComponentKey, ConfiguredProps, Error, ExternalUserId, PageRequest, Result, SecretString,
    Timestamp, UnknownFields, WebhookId, WorkflowEndpoint,
};

/// A registry trigger deployment request.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DeployTriggerRequest {
    /// Trigger component key.
    pub id: ComponentKey,
    /// Component version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// External user partition.
    pub external_user_id: ExternalUserId,
    /// Configured trigger values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configured_props: Option<ConfiguredProps>,
    /// Dynamic property context ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_props_id: Option<String>,
    /// Workflow endpoint receiving events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_id: Option<WorkflowEndpoint>,
    /// Webhook destination receiving events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<SecretString>,
    /// Whether deploy hooks may emit an event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emit_on_deploy: Option<bool>,
}

/// A partial deployed trigger update.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct UpdateTriggerRequest {
    /// Active state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Configured trigger values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configured_props: Option<ConfiguredProps>,
    /// Display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether deploy hooks may emit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emit_on_deploy: Option<bool>,
}

/// Deployed trigger list query.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ListDeployedTriggersRequest {
    /// Cursor and limit.
    #[serde(flatten)]
    pub page: PageRequest,
    /// Emitter type filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emitter_type: Option<EmitterType>,
}

/// One deployed-trigger event.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EmittedEvent {
    /// Event payload.
    pub e: Value,
    /// Event kind.
    pub k: String,
    /// Raw epoch-millisecond timestamp.
    pub ts: i64,
    /// Event identifier.
    pub id: String,
    /// Optional event summary.
    #[serde(default)]
    pub sum: Option<String>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

impl EmittedEvent {
    /// Converts the raw event timestamp to UTC.
    pub fn timestamp(&self) -> Result<DateTime<Utc>> {
        Timestamp::from_millis(self.ts).map(|timestamp| timestamp.0)
    }
}

/// One deployed-trigger webhook destination.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TriggerWebhook {
    /// Webhook identifier.
    pub id: WebhookId,
    /// Destination URL.
    pub url: SecretString,
    /// Signing key when returned.
    #[serde(default)]
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
    pub webhook_urls: Vec<SecretString>,
    /// Destination metadata.
    #[serde(default)]
    pub webhooks: Vec<TriggerWebhook>,
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

/// Trigger event history query.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ListTriggerEventsRequest {
    /// Maximum event count.
    pub limit: u16,
}

impl ListTriggerEventsRequest {
    /// Creates a bounded event history request.
    pub fn new(limit: u16) -> Result<Self> {
        if !(1..=100).contains(&limit) {
            return Err(Error::OutOfRange {
                field: "limit",
                minimum: 1,
                maximum: 100,
            });
        }
        Ok(Self { limit })
    }
}

/// Delete deployed trigger options.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct DeleteTriggerRequest {
    /// Ignore hook errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_hook_errors: Option<bool>,
}

/// Trigger webhook replacement request.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct UpdateTriggerWebhooksRequest {
    /// Destination URLs.
    pub webhook_urls: Vec<SecretString>,
}

/// Trigger workflow replacement request.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct UpdateTriggerWorkflowsRequest {
    /// Workflow endpoint values.
    pub workflow_ids: Vec<WorkflowEndpoint>,
}

/// Trigger deployment response.
pub type DeployTriggerResponse = crate::Data<Emitter>;

/// Deployed trigger response.
pub type GetTriggerResponse = crate::Data<Emitter>;

/// Deployed trigger list response.
pub type GetTriggersResponse = crate::PageEnvelope<Emitter>;

/// Deployed trigger event response.
pub type GetTriggerEventsResponse = crate::Data<Vec<EmittedEvent>>;

/// Deployed trigger webhook response.
pub type GetTriggerWebhooksResponse = TriggerWebhooks;

/// Deployed trigger workflow response.
pub type GetTriggerWorkflowsResponse = TriggerWorkflows;
