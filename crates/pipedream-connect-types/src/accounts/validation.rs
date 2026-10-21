//! Connect token validation response.

use serde::{Deserialize, Serialize};

use crate::{App, Environment, ProjectId, UnknownFields};

/// Connect token validation response.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ValidateTokenResponse {
    /// Resolved app metadata.
    #[serde(default)]
    pub app: Option<App>,
    /// Validation error text.
    #[serde(default)]
    pub error: Option<String>,
    /// Error redirect URI.
    #[serde(default)]
    pub error_redirect_uri: Option<String>,
    /// OAuth app identifier.
    #[serde(default)]
    pub oauth_app_id: Option<String>,
    /// Whether Pipedream selected a Workday official client.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oauth_app_workday_official: Option<bool>,
    /// Project app name.
    #[serde(default)]
    pub project_app_name: Option<String>,
    /// Project environment.
    #[serde(default)]
    pub project_environment: Option<Environment>,
    /// Project identifier.
    #[serde(default)]
    pub project_id: Option<ProjectId>,
    /// Project support email.
    #[serde(default)]
    pub project_support_email: Option<String>,
    /// Validation result.
    pub success: bool,
    /// Success redirect URI.
    #[serde(default)]
    pub success_redirect_uri: Option<String>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}
