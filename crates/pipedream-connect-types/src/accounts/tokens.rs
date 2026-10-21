//! Connect token creation and response models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use url::Url;

use crate::{
    ConnectToken, Error, ExternalUserId, ProjectId, Result, SecretString, UnknownFields, UrlField,
    ValidationField,
};

/// Hosted Connect token creation request.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreateConnectTokenRequest {
    /// Project identifier when the body is sent without a path context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<ProjectId>,
    /// External user that owns the session.
    pub external_user_id: ExternalUserId,
    /// Allowed browser origins.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_origins: Vec<String>,
    /// HTTPS success redirect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_redirect_uri: Option<String>,
    /// HTTPS error redirect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_redirect_uri: Option<String>,
    /// Token lifetime in seconds.
    #[serde(
        default = "default_expires_in",
        deserialize_with = "deserialize_expires_in"
    )]
    pub expires_in: u32,
    /// Requested Connect scope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// Connection webhook URI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_uri: Option<String>,
    /// Whether progressive scopes are allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_progressive_scopes: Option<bool>,
}

fn default_expires_in() -> u32 {
    14_400
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
    pub fn with_expires_in(mut self, expires_in: u32) -> Result<Self> {
        validate_expires_in(expires_in)?;
        self.expires_in = expires_in;
        Ok(self)
    }

    /// Sets the project identifier carried by the body when needed.
    pub fn with_project_id(mut self, project_id: ProjectId) -> Self {
        self.project_id = Some(project_id);
        self
    }

    /// Sets and validates an HTTPS success redirect URI.
    pub fn with_success_redirect_uri(mut self, uri: impl Into<String>) -> Result<Self> {
        self.success_redirect_uri = Some(uri.into());
        self.validate()?;
        Ok(self)
    }

    /// Sets and validates an HTTPS error redirect URI.
    pub fn with_error_redirect_uri(mut self, uri: impl Into<String>) -> Result<Self> {
        self.error_redirect_uri = Some(uri.into());
        self.validate()?;
        Ok(self)
    }

    /// Sets and validates an HTTPS connection webhook URI.
    pub fn with_webhook_uri(mut self, uri: impl Into<String>) -> Result<Self> {
        self.webhook_uri = Some(uri.into());
        self.validate()?;
        Ok(self)
    }

    /// Validates token lifetime and all configured URI fields.
    pub fn validate(&self) -> Result<()> {
        validate_expires_in(self.expires_in)?;
        for (field, value) in [
            (UrlField::SuccessRedirectUri, &self.success_redirect_uri),
            (UrlField::ErrorRedirectUri, &self.error_redirect_uri),
            (UrlField::WebhookUri, &self.webhook_uri),
        ] {
            if let Some(value) = value {
                let parsed = match Url::parse(value) {
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

fn validate_expires_in(expires_in: u32) -> Result<()> {
    if !(1..=14_400).contains(&expires_in) {
        return Err(Error::OutOfRange {
            field: ValidationField::ExpiresIn,
            minimum: 1,
            maximum: 14_400,
        });
    }
    Ok(())
}

fn deserialize_expires_in<'de, D>(deserializer: D) -> std::result::Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let expires_in = u32::deserialize(deserializer)?;
    match validate_expires_in(expires_in) {
        Ok(()) => Ok(expires_in),
        Err(error) => Err(serde::de::Error::custom(error)),
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

#[cfg(test)]
#[path = "_tests_/tokens_tests.rs"]
mod tokens_tests;
