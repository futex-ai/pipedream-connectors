//! Connected account, external user, and Connect token models.

use crate::{
    AccountId, App, AppSlug, ConnectToken, Error, ExternalUserId, OauthAppId, PageRequest,
    ProjectId, SecretJson, SecretString, UnknownFields,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    pub healthy: bool,
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
    pub authorized_scopes: Vec<String>,
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
    pub cfmap_json: String,
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
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
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
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ListUsersRequest {
    /// Cursor and limit.
    #[serde(flatten)]
    pub page: PageRequest,
    /// Search query sent as `q`.
    #[serde(rename = "q", skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
}

/// Hosted Connect token creation request.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CreateConnectTokenRequest {
    /// Project identifier when the body is sent without a path context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<ProjectId>,
    /// External user that owns the session.
    pub external_user_id: ExternalUserId,
    /// Allowed browser origins.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub allowed_origins: Vec<String>,
    /// HTTPS success redirect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_redirect_uri: Option<SecretString>,
    /// HTTPS error redirect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_redirect_uri: Option<SecretString>,
    /// Token lifetime in seconds.
    pub expires_in: u32,
    /// Requested Connect scope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Connection webhook URI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_uri: Option<SecretString>,
    /// Whether progressive scopes are allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_progressive_scopes: Option<bool>,
}

impl CreateConnectTokenRequest {
    /// Creates a request with the documented four-hour default lifetime.
    pub fn new(external_user_id: ExternalUserId) -> Self {
        Self {
            project_id: None,
            external_user_id,
            allowed_origins: Vec::new(),
            success_redirect_uri: None,
            error_redirect_uri: None,
            expires_in: 14_400,
            scope: None,
            webhook_uri: None,
            allow_progressive_scopes: None,
        }
    }

    /// Sets a validated lifetime in seconds.
    pub fn with_expires_in(mut self, expires_in: u32) -> crate::Result<Self> {
        if !(1..=14_400).contains(&expires_in) {
            return Err(Error::OutOfRange {
                field: "expires_in",
                minimum: 1,
                maximum: 14_400,
            });
        }
        self.expires_in = expires_in;
        Ok(self)
    }

    /// Sets the project identifier carried by the body when needed.
    pub fn with_project_id(mut self, project_id: ProjectId) -> Self {
        self.project_id = Some(project_id);
        self
    }

    /// Sets and validates an HTTPS success redirect URI.
    pub fn with_success_redirect_uri(mut self, uri: impl Into<String>) -> crate::Result<Self> {
        self.success_redirect_uri = Some(SecretString::new(uri));
        self.validate()?;
        Ok(self)
    }

    /// Sets and validates an HTTPS error redirect URI.
    pub fn with_error_redirect_uri(mut self, uri: impl Into<String>) -> crate::Result<Self> {
        self.error_redirect_uri = Some(SecretString::new(uri));
        self.validate()?;
        Ok(self)
    }

    /// Sets and validates an HTTPS connection webhook URI.
    pub fn with_webhook_uri(mut self, uri: impl Into<String>) -> crate::Result<Self> {
        self.webhook_uri = Some(SecretString::new(uri));
        self.validate()?;
        Ok(self)
    }

    /// Validates token lifetime and all configured URI fields.
    pub fn validate(&self) -> crate::Result<()> {
        if !(1..=14_400).contains(&self.expires_in) {
            return Err(Error::OutOfRange {
                field: "expires_in",
                minimum: 1,
                maximum: 14_400,
            });
        }
        for (field, value) in [
            ("success_redirect_uri", &self.success_redirect_uri),
            ("error_redirect_uri", &self.error_redirect_uri),
            ("webhook_uri", &self.webhook_uri),
        ] {
            if let Some(value) = value {
                let parsed = match url::Url::parse(value.expose_secret()) {
                    Ok(parsed) => parsed,
                    Err(_) => return Err(Error::InvalidHttpsUrl { field }),
                };
                if parsed.scheme() != "https" || parsed.host_str().is_none() {
                    return Err(Error::InvalidHttpsUrl { field });
                }
            }
        }
        Ok(())
    }
}

/// Connect token creation response.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ConnectTokenResponse {
    /// Connect Link URL.
    pub connect_link_url: SecretString,
    /// Token expiration.
    pub expires_at: DateTime<Utc>,
    /// Short-lived token.
    pub token: ConnectToken,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

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
    #[serde(default)]
    pub oauth_app_workday_official: bool,
    /// Project app name.
    #[serde(default)]
    pub project_app_name: Option<String>,
    /// Project environment.
    #[serde(default)]
    pub project_environment: Option<crate::Environment>,
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
