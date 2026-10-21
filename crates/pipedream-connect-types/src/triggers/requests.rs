//! Trigger deployment models and requests.

use serde::{Deserialize, Serialize};

use crate::{
    ComponentKey, ConfiguredProps, EmitterType, ExternalUserId, PageRequest, WorkflowEndpoint,
};

/// A registry trigger deployment request.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    pub webhook_url: Option<String>,
    /// Whether deploy hooks may emit an event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emit_on_deploy: Option<bool>,
}

/// A partial deployed trigger update.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ListDeployedTriggersRequest {
    /// External user that owns the deployed triggers.
    pub external_user_id: ExternalUserId,
    /// Cursor and limit.
    #[serde(flatten)]
    pub page: PageRequest,
    /// Emitter type filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emitter_type: Option<EmitterType>,
}
