//! Configuration and action execution request and response models.

mod observations;
pub use observations::*;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    ComponentKey, ConfigurableProp, ConfiguredProps, ExternalUserId, PropName, UnknownFields,
};

/// The stage that caused a provider execution failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorOrigin {
    /// Component code failed.
    ComponentCode,
    /// The upstream API returned an error.
    UpstreamApi,
    /// Network I/O failed.
    NetworkIo,
    /// Response parsing failed.
    ResponseParsing,
    /// A provider origin added after this SDK version.
    Other(String),
}

impl Serialize for ErrorOrigin {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = match self {
            Self::ComponentCode => "component_code",
            Self::UpstreamApi => "upstream_api",
            Self::NetworkIo => "network_io",
            Self::ResponseParsing => "response_parsing",
            Self::Other(value) => value.as_str(),
        };
        serializer.serialize_str(value)
    }
}

impl<'de> Deserialize<'de> for ErrorOrigin {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "component_code" => Self::ComponentCode,
            "upstream_api" => Self::UpstreamApi,
            "network_io" => Self::NetworkIo,
            "response_parsing" => Self::ResponseParsing,
            _ => Self::Other(value),
        })
    }
}

/// Details about the upstream calls made by an execution.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ExecutionAttribution {
    /// Failure stage.
    pub origin: ErrorOrigin,
    /// Number of upstream calls.
    #[serde(default)]
    pub network_calls_count: Option<u64>,
    /// Last upstream call metadata.
    #[serde(default)]
    pub last_call: Option<NetworkCall>,
    /// Unknown attribution fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Metadata for the last upstream call.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NetworkCall {
    /// Upstream client name.
    #[serde(default)]
    pub client: Option<String>,
    /// Upstream HTTP method.
    #[serde(default)]
    pub method: Option<String>,
    /// Upstream host.
    #[serde(default)]
    pub host: Option<String>,
    /// Upstream status.
    #[serde(default)]
    pub status: Option<u16>,
    /// Upstream duration in milliseconds.
    #[serde(default)]
    pub duration_ms: Option<f64>,
    /// Unknown call fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// A structured provider execution failure.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ExecutionError {
    /// Failure attribution.
    #[serde(default)]
    pub attribution: Option<ExecutionAttribution>,
    /// Provider error name.
    #[serde(default)]
    pub name: Option<String>,
    /// Provider error message.
    #[serde(default)]
    pub message: Option<String>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Request to configure one component property.
#[derive(Clone, Debug, PartialEq, Serialize)]
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
    pub page: Option<u32>,
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
    pub options: Option<Vec<crate::PropOption>>,
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
    #[serde(default)]
    pub errors: Vec<String>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Request to reload a component's dynamic properties.
#[derive(Clone, Debug, PartialEq, Serialize)]
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
    #[serde(default)]
    pub id: Option<String>,
    /// Updated property definitions.
    #[serde(rename = "configurableProps", default)]
    pub configurable_props: Vec<ConfigurableProp>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Response from reloading dynamic properties.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ReloadPropsResponse {
    /// Execution observations.
    #[serde(default)]
    pub observations: Vec<Observation>,
    /// Provider errors.
    #[serde(default)]
    pub errors: Vec<String>,
    /// Updated dynamic properties.
    #[serde(rename = "dynamicProps", default)]
    pub dynamic_props: Option<DynamicProps>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Request to execute an action.
#[derive(Clone, Debug, PartialEq, Serialize)]
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
    pub stash_id: Option<String>,
}

/// Action execution result.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct RunActionResponse {
    /// Values exported by action steps.
    #[serde(default)]
    pub exports: Option<Value>,
    /// Execution observations.
    #[serde(default)]
    pub os: Vec<Observation>,
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
