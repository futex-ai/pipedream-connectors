//! Connected accounts and external users.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    AccountId, App, AppSlug, ConnectToken, ExternalUserId, OauthAppId, PageRequest, SecretJson,
    SecretString, UnknownFields,
};

/// A connected account returned by the API.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Account {
    /// Account identifier.
    pub id: AccountId,
    /// Optional custom account name.
    #[serde(default)]
    pub name: Option<String>,
    /// External user partition.
    #[serde(default)]
    pub external_id: Option<ExternalUserId>,
    /// Whether credentials are healthy.
    #[serde(default)]
    pub healthy: Option<bool>,
    /// Whether the account is inactive.
    #[serde(default)]
    pub dead: Option<bool>,
    /// App metadata.
    #[serde(default)]
    pub app: Option<App>,
    /// Creation time.
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    /// Last update time.
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    /// OAuth scopes.
    #[serde(default)]
    pub authorized_scopes: Option<Vec<String>>,
    /// Credential object, redacted in diagnostics.
    #[serde(default)]
    pub credentials: Option<SecretJson>,
    /// Credential expiry.
    #[serde(default)]
    pub expires_at: Option<DateTime<Utc>>,
    /// Provider health error.
    #[serde(default)]
    pub error: Option<String>,
    /// Last credential refresh.
    #[serde(default)]
    pub last_refreshed_at: Option<DateTime<Utc>>,
    /// Next credential refresh.
    #[serde(default)]
    pub next_refresh_at: Option<DateTime<Utc>>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Request to create or reconnect an account.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CreateAccountRequest {
    /// Provider app slug.
    pub app_slug: AppSlug,
    /// Custom field mapping sent as JSON.
    pub cfmap_json: SecretString,
    /// Connect token used for account authorization.
    pub connect_token: ConnectToken,
    /// Optional display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Existing account to reconnect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<AccountId>,
}

/// Account list query.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ListAccountsRequest {
    /// Cursor and limit.
    #[serde(flatten)]
    pub page: PageRequest,
    /// Optional external-user filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_user_id: Option<ExternalUserId>,
    /// App filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<AppSlug>,
    /// Custom OAuth client filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_app_id: Option<OauthAppId>,
    /// Include credential values in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_credentials: Option<bool>,
}

/// External user record.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ExternalUser {
    /// Pipedream external-user identifier.
    pub id: String,
    /// Host-provided external user identifier.
    pub external_id: ExternalUserId,
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

/// External-user list query.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ListUsersRequest {
    /// Cursor and limit.
    #[serde(flatten)]
    pub page: PageRequest,
    /// Search query sent as `q`.
    #[serde(rename = "q", skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
}
