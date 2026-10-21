//! Execution models and requests.

use serde::{Deserialize, Serialize};

use crate::{UnknownFields, string_enum};

string_enum! {
    /// The stage that caused a provider execution failure.
    pub enum ErrorOrigin {
        /// Component code failed.
        ComponentCode => "component_code",
        /// The upstream API returned an error.
        UpstreamApi => "upstream_api",
        /// Network I/O failed.
        NetworkIo => "network_io",
        /// Response parsing failed.
        ResponseParsing => "response_parsing",
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
