//! Queries for account retrieval and Connect token validation.

use serde::{Deserialize, Serialize};

use crate::{AccountId, AppSlug, ExternalUserId, OauthAppId};

/// Optional tenant filters for account creation.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct CreateAccountQuery {
    /// External user that owns the new account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_user_id: Option<ExternalUserId>,
    /// Custom OAuth client used by the account.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_app_id: Option<OauthAppId>,
}

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
    pub app_id: AppSlug,
    /// Optional account to reconnect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<AccountId>,
    /// Optional custom OAuth app.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_app_id: Option<OauthAppId>,
}
