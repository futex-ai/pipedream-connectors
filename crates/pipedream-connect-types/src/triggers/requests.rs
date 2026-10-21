//! Trigger deployment models and requests.

use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    ComponentKey, ConfiguredProps, EmitterType, Error, ExternalUserId, PageRequest, Result,
    WorkflowEndpoint,
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

impl DeployTriggerRequest {
    /// Validates the optional trigger callback URL.
    pub fn validate(&self) -> Result<()> {
        if let Some(url) = &self.webhook_url {
            validate_trigger_webhook_url(url)?;
        }
        Ok(())
    }
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

pub(super) fn validate_trigger_webhook_url(value: &str) -> Result<()> {
    let parsed = Url::parse(value).map_err(|_| Error::InvalidTriggerWebhookUrl)?;
    let loopback_http = parsed.scheme() == "http"
        && matches!(parsed.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if value.trim() != value
        || value.contains('\\')
        || value.chars().any(char::is_control)
        || !(parsed.scheme() == "https" || loopback_http)
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.fragment().is_some()
    {
        return Err(Error::InvalidTriggerWebhookUrl);
    }
    Ok(())
}
