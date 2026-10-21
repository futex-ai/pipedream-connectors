//! Project, OAuth, webhook, and usage administration models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::{App, AppSlug, OauthAppId, ProjectId, SecretString, UnknownFields};

/// A Connect project.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Project {
    /// Project identifier.
    pub id: ProjectId,
    /// Display name.
    pub name: String,
    /// Connect app name.
    #[serde(default)]
    pub app_name: Option<String>,
    /// Support email.
    #[serde(default)]
    pub support_email: Option<String>,
    /// Whether key-based account checks are required.
    #[serde(default)]
    pub connect_require_key_auth_test: bool,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Request to create a project.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CreateProjectRequest {
    /// Project display name.
    pub name: String,
    /// Connect app name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
    /// Support email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_email: Option<String>,
    /// Whether key-based account checks are required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_require_key_auth_test: Option<bool>,
}

/// Partial project update.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct UpdateProjectRequest {
    /// New display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New Connect app name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
    /// New support email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_email: Option<String>,
    /// New key-auth test policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_require_key_auth_test: Option<bool>,
}

/// Project logo update body.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct UpdateProjectLogoRequest {
    /// Data URI containing a base64 image.
    pub logo: String,
}

/// One app enabled for a project environment.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ProjectInfoApp {
    /// Custom OAuth app identifier, when present.
    #[serde(default)]
    pub id: Option<OauthAppId>,
    /// Provider app slug.
    pub name_slug: AppSlug,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Apps enabled for a project environment.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ProjectInfo {
    /// Enabled apps.
    pub apps: Vec<ProjectInfoApp>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// A custom OAuth client returned by Connect.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct OauthApp {
    /// OAuth app identifier.
    pub id: OauthAppId,
    /// Upstream OAuth client identifier.
    pub client_id: String,
    /// Human-readable app name inherited from the app metadata.
    #[serde(default)]
    pub name: Option<String>,
    /// App authentication mode.
    #[serde(default)]
    pub auth_type: Option<crate::AppAuthType>,
    /// App description.
    #[serde(default)]
    pub description: Option<String>,
    /// App logo URL.
    #[serde(default)]
    pub img_src: Option<String>,
    /// JSON-encoded custom field metadata.
    #[serde(default)]
    pub custom_fields_json: Option<String>,
    /// App categories.
    #[serde(default)]
    pub categories: Vec<String>,
    /// App popularity weight.
    #[serde(default)]
    pub featured_weight: f64,
    /// Named OAuth scope profiles.
    #[serde(default)]
    pub scope_profiles: Vec<crate::ScopeProfile>,
    /// Provider app slug when present in the merged app metadata.
    #[serde(default)]
    pub name_slug: Option<AppSlug>,
    /// Requested OAuth scopes.
    #[serde(default)]
    pub scopes: Vec<String>,
    /// Additional user-granted scopes.
    #[serde(default)]
    pub additional_scopes: Option<Vec<String>>,
    /// Upstream redirect URI.
    #[serde(default)]
    pub redirect_uri: Option<String>,
    /// Resource owner identifier.
    pub owner_id: String,
    /// Creation time.
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Request to create a custom OAuth client.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CreateOauthAppRequest {
    /// Provider app slug or ID.
    pub app: String,
    /// Upstream OAuth client ID.
    pub client_id: String,
    /// Upstream OAuth client secret.
    pub client_secret: SecretString,
    /// Display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Base scopes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
    /// Additional scopes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_scopes: Option<Vec<String>>,
}

/// Partial custom OAuth client update.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct UpdateOauthAppRequest {
    /// Display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Upstream OAuth client ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    /// Upstream OAuth client secret.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<SecretString>,
    /// Base scopes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
    /// Additional scopes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_scopes: Option<Vec<String>>,
}

/// A named app configuration override.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AppOverride {
    /// Override identifier.
    pub id: String,
    /// Override name.
    pub name: String,
    /// App metadata selected by the override.
    pub app: App,
    /// Selected custom OAuth app.
    #[serde(default)]
    pub oauth_app_id: Option<OauthAppId>,
    /// Predefined custom fields.
    #[serde(default)]
    pub cfmap: BTreeMap<String, Value>,
    /// Owner identifier.
    pub owner_id: String,
    /// Creation time.
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Request to create an app override.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CreateAppOverrideRequest {
    /// App slug or ID.
    pub app: String,
    /// Override name.
    pub name: String,
    /// Selected custom OAuth app.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_app_id: Option<OauthAppId>,
    /// Predefined custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cfmap: Option<BTreeMap<String, Value>>,
}

/// Partial app override update.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct UpdateAppOverrideRequest {
    /// New override name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New selected OAuth app.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_app_id: Option<Option<OauthAppId>>,
    /// New predefined fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cfmap: Option<BTreeMap<String, Value>>,
}

/// Connect usage for one time window.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct UsageRecord {
    /// Total credits used.
    pub credits_used: i64,
    /// Action credits.
    #[serde(default)]
    pub action_run_credits_used: Option<i64>,
    /// Proxy credits.
    #[serde(default)]
    pub proxy_credits_used: Option<i64>,
    /// Trigger emission credits.
    #[serde(default)]
    pub source_emit_credits_used: Option<i64>,
    /// Start timestamp in epoch seconds.
    pub usage_start_ts: i64,
    /// End timestamp in epoch seconds.
    pub usage_end_ts: i64,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

mod tokens;
pub use tokens::*;

mod webhooks;
pub use webhooks::*;
