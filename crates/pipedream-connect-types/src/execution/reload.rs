//! Execution models and requests.

use serde::{Deserialize, Serialize};

use crate::{
    ComponentKey, ConfigurableProp, ConfiguredProps, ExternalUserId, Observation, UnknownFields,
};

/// Request to reload a component's dynamic properties.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReloadPropsRequest {
    /// Component key.
    pub id: ComponentKey,
    /// Component version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// External user partition.
    pub external_user_id: ExternalUserId,
    /// Whether to wait for completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocking: Option<bool>,
    /// Current configured values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configured_props: Option<ConfiguredProps>,
    /// Dynamic property context ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_props_id: Option<String>,
}

/// Updated dynamic properties.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct DynamicProps {
    /// Dynamic property context ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Updated property definitions.
    #[serde(
        rename = "configurableProps",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub configurable_props: Option<Vec<ConfigurableProp>>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Response from reloading dynamic properties.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ReloadPropsResponse {
    /// Execution observations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observations: Option<Vec<Observation>>,
    /// Provider errors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<String>>,
    /// Updated dynamic properties.
    #[serde(
        rename = "dynamicProps",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub dynamic_props: Option<DynamicProps>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}
