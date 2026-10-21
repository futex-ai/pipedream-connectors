//! Rate-limit and OAuth token models.

use serde::{Deserialize, Serialize};

use crate::{SecretString, UnknownFields, string_enum};

string_enum! {
    /// OAuth grant type accepted by Pipedream.
    pub enum GrantType {
        /// Client credentials grant.
        ClientCredentials => "client_credentials",
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
    /// Size of the rate-limit window in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_size_seconds: Option<u64>,
    /// Number of requests allowed in the window.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests_per_window: Option<u64>,
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
