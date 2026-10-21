//! Queries for account retrieval and Connect token validation.

use serde::{Deserialize, Serialize};

use crate::{AccountId, AppSlug, OauthAppId};

/// Account retrieval query.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct GetAccountRequest {
    /// Include account credentials when the app permits it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_credentials: Option<bool>,
}

/// Query for the accounts of one external user.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ListUserAccountsRequest {
    /// Include credentials when the app permits it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_credentials: Option<bool>,
    /// Provider app filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<AppSlug>,
}

/// Connect token validation query.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ValidateTokenRequest {
    /// Provider app ID or slug.
    pub app_id: String,
    /// Optional account to reconnect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<AccountId>,
    /// Optional custom OAuth app.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_app_id: Option<OauthAppId>,
}
