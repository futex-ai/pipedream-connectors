//! Generic wire envelopes shared by Connect operations.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    Account, App, AppCategory, AppOverride, Component, ExternalUser, OauthApp, Page, Project,
    UnknownFields, UsageRecord, Webhook, WebhookWithOptionalSigningKey, WebhookWithSigningKey,
};

/// A response containing one data value.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Data<T> {
    /// Response data.
    pub data: T,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// The documented Connect error envelope.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ErrorResponse {
    /// Human-readable provider error.
    pub error: ErrorMessage,
    /// Provider error code.
    #[serde(default)]
    pub code: Option<String>,
    /// Provider error details.
    #[serde(default)]
    pub details: Option<Value>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// App category list response.
pub type ListAppCategoriesResponse = Vec<AppCategory>;

/// Project list response.
pub type ListProjectsResponse = Page<Project>;

/// OAuth app list response.
pub type ListOauthAppsResponse = Page<OauthApp>;

/// App override list response.
pub type ListAppOverridesResponse = Page<AppOverride>;

/// Account list response.
pub type ListAccountsResponse = Page<Account>;

/// User list response.
pub type GetUsersResponse = Page<ExternalUser>;

/// App list response.
pub type ListAppsResponse = Page<App>;

/// Component list response.
pub type GetComponentsResponse = Page<Component>;

/// Usage list response.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ConnectUsageResponse {
    /// Usage records.
    pub data: Vec<UsageRecord>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Paged account response returned by account list operations.
pub type GetAccountsResponse = Page<Account>;

/// Bare account returned by account retrieval.
pub type GetAccountResponse = Account;

/// Bare account returned by account creation.
pub type CreateAccountResponse = Account;

/// One app response.
pub type GetAppResponse = Data<App>;

/// One component response.
pub type GetComponentResponse = Data<Component>;

/// One category response.
pub type GetAppCategoryResponse = AppCategory;

/// One project webhook response.
pub type GetWebhookResponse = Data<Option<Webhook>>;

/// One webhook response that includes its signing key.
pub type GetWebhookWithSigningKeyResponse = Data<WebhookWithSigningKey>;

/// Project webhook update response.
pub type SetWebhookResponse = Data<WebhookWithOptionalSigningKey>;

/// Provider error text or a structured, app-defined error object.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ErrorMessage {
    /// Human-readable error text.
    Text(String),
    /// App-defined error JSON.
    Json(Value),
}

#[cfg(test)]
#[path = "_tests_/wire_tests.rs"]
mod wire_tests;

#[cfg(test)]
#[path = "_tests_/requests_tests.rs"]
mod requests_tests;
