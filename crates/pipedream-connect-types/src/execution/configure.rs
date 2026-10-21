//! Execution models and requests.

use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};

use crate::{
    ComponentKey, ConfiguredProps, ExternalUserId, Observation, PropName, PropOption, UnknownFields,
};

/// Request to configure one component property.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConfigurePropRequest {
    /// Component key.
    pub id: ComponentKey,
    /// Component version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// External user partition.
    pub external_user_id: ExternalUserId,
    /// Property to configure.
    pub prop_name: PropName,
    /// Whether to wait for completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocking: Option<bool>,
    /// Current configured values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configured_props: Option<ConfiguredProps>,
    /// Dynamic property context ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_props_id: Option<String>,
    /// Provider option page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<Number>,
    /// Previous provider context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_context: Option<Value>,
    /// Remote option query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
}

/// Response from configuring one property.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ConfigurePropResponse {
    /// Labelled or scalar options.
    #[serde(default)]
    pub options: Option<Vec<PropOption>>,
    /// String-only options.
    #[serde(rename = "stringOptions", default)]
    pub string_options: Option<Vec<String>>,
    /// Execution observations.
    #[serde(default)]
    pub observations: Option<Vec<Observation>>,
    /// Provider context for the next call.
    #[serde(default)]
    pub context: Option<Value>,
    /// Provider errors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}
