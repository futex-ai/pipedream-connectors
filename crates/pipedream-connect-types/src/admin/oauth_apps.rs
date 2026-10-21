//! Project administration models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{AppAuthType, AppSlug, OauthAppId, ScopeProfile, SecretString, UnknownFields};

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
    pub auth_type: Option<AppAuthType>,
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
    pub categories: Option<Vec<String>>,
    /// App popularity weight.
    #[serde(default)]
    pub featured_weight: Option<f64>,
    /// Named OAuth scope profiles.
    #[serde(default)]
    pub scope_profiles: Option<Vec<ScopeProfile>>,
    /// Provider app slug when present in the merged app metadata.
    #[serde(default)]
    pub name_slug: Option<AppSlug>,
    /// Requested OAuth scopes.
    #[serde(default)]
    pub scopes: Option<Vec<String>>,
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
