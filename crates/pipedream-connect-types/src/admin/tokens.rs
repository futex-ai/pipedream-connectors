//! Rate-limit and OAuth token models.

use serde::{Deserialize, Serialize};

use crate::{SecretString, UnknownFields};

/// OAuth grant type accepted by Pipedream.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub enum GrantType {
    /// Client credentials grant.
    ClientCredentials,
    /// A provider grant added after this SDK version.
    Other(String),
}

impl<'de> Deserialize<'de> for GrantType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "client_credentials" => Self::ClientCredentials,
            _ => Self::Other(value),
        })
    }
}

/// OAuth client-credentials token request.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CreateOauthTokenRequest {
    /// OAuth grant type.
    pub grant_type: GrantType,
    /// OAuth client identifier.
    pub client_id: String,
    /// OAuth client secret.
    pub client_secret: SecretString,
    /// Optional space-separated scope string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

/// Request for a custom rate-limit token.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct RateLimitRequest {
    /// Optional requested token name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Unknown request fields accepted for forward compatibility.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Custom rate-limit token returned by Connect.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RateLimitToken {
    /// Token value.
    pub token: SecretString,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Data returned by the OAuth token endpoint.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct OAuthTokenResponse {
    /// OAuth bearer value.
    pub access_token: SecretString,
    /// Token type.
    pub token_type: String,
    /// Lifetime in seconds.
    pub expires_in: u32,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Alias matching the OpenAPI response schema name.
pub type CreateOauthTokenResponse = OAuthTokenResponse;
