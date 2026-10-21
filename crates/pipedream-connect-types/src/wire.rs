//! Generic wire envelopes shared by Connect operations.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    Account, AppCategory, AppOverride, Component, ExternalUser, OauthApp, PageInfo, Project,
    UnknownFields, UsageRecord, Webhook,
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

/// A response containing a page of data.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct PageEnvelope<T> {
    /// Page records.
    pub data: Vec<T>,
    /// Cursor metadata.
    pub page_info: PageInfo,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// The documented Connect error envelope.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ErrorResponse {
    /// Human-readable provider error.
    pub error: String,
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
pub type ListProjectsResponse = PageEnvelope<Project>;

/// OAuth app list response.
pub type ListOauthAppsResponse = PageEnvelope<OauthApp>;

/// App override list response.
pub type ListAppOverridesResponse = PageEnvelope<AppOverride>;

/// Account list response.
pub type ListAccountsResponse = PageEnvelope<Account>;

/// User list response.
pub type GetUsersResponse = PageEnvelope<ExternalUser>;

/// App list response.
pub type ListAppsResponse = PageEnvelope<crate::App>;

/// Component list response.
pub type GetComponentsResponse = PageEnvelope<Component>;

/// Usage list response.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ConnectUsageResponse {
    /// Usage records.
    pub data: Vec<UsageRecord>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// One account response.
pub type GetAccountsResponse = Data<Account>;

/// One app response.
pub type GetAppResponse = Data<crate::App>;

/// One component response.
pub type GetComponentResponse = Data<Component>;

/// One category response.
pub type GetAppCategoryResponse = AppCategory;

/// One project webhook response.
pub type GetWebhookResponse = Data<Option<Webhook>>;

/// One webhook response that includes its signing key.
pub type GetWebhookWithSigningKeyResponse = Data<crate::WebhookWithSigningKey>;

/// Project webhook update response.
pub type SetWebhookResponse = Data<crate::WebhookWithOptionalSigningKey>;
