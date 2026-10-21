//! Execution models and requests.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    ComponentKey, ConfiguredProps, ExternalUserId, Observation, StashReference, UnknownFields,
};

/// Request to execute an action.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunActionRequest {
    /// Action component key.
    pub id: ComponentKey,
    /// Component version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// External user partition.
    pub external_user_id: ExternalUserId,
    /// Configured action values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configured_props: Option<ConfiguredProps>,
    /// Dynamic property context ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_props_id: Option<String>,
    /// Optional File Stash identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stash_id: Option<StashReference>,
}

/// Action execution result.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct RunActionResponse {
    /// Values exported by action steps.
    #[serde(default)]
    pub exports: Option<Value>,
    /// Execution observations.
    #[serde(default)]
    pub os: Option<Vec<Observation>>,
    /// Action return value.
    #[serde(default)]
    pub ret: Option<Value>,
    /// File Stash identifier.
    #[serde(default)]
    pub stash_id: Option<String>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}
