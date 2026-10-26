//! Project administration models.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::{App, AppOverrideId, OauthAppId, Timestamp, UnknownFields};

/// A named app configuration override.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AppOverride {
    /// Override identifier.
    pub id: AppOverrideId,
    /// Override name.
    pub name: String,
    /// App metadata selected by the override.
    pub app: App,
    /// Selected custom OAuth app.
    #[serde(default)]
    pub oauth_app_id: Option<OauthAppId>,
    /// Predefined custom fields.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cfmap: Option<BTreeMap<String, Value>>,
    /// Owner identifier.
    pub owner_id: String,
    /// Creation time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<Timestamp>,
    /// Last update time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<Timestamp>,
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
    /// New selected OAuth app, including explicit null to clear the selection.
    #[serde(
        default,
        deserialize_with = "nullable_oauth_app",
        skip_serializing_if = "Option::is_none"
    )]
    pub oauth_app_id: Option<Option<OauthAppId>>,
    /// New predefined fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cfmap: Option<BTreeMap<String, Value>>,
}

fn nullable_oauth_app<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<Option<OauthAppId>>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<OauthAppId>::deserialize(deserializer).map(Some)
}
