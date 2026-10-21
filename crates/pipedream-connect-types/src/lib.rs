//! Typed, lossless data models for the Pipedream Connect API.

#![warn(unreachable_pub)]

mod accounts;
mod admin;
mod epoch_millis;
mod error;
mod execution;
mod ids;
#[cfg(test)]
mod option_epoch_millis;
mod pagination;
mod props;
mod registry;
mod secret;
mod shared;
mod triggers;
mod wire;

pub use accounts::{
    Account, ConnectTokenResponse, CreateAccountRequest, CreateConnectTokenRequest, ExternalUser,
    GetAccountRequest, ListAccountsRequest, ListUserAccountsRequest, ListUsersRequest,
    ValidateTokenRequest, ValidateTokenResponse,
};
pub use admin::{
    AppOverride, CreateAppOverrideRequest, CreateOauthAppRequest, CreateOauthTokenRequest,
    CreateOauthTokenResponse, CreateProjectRequest, DownloadFileStashRequest, GrantType,
    ListAppOverridesRequest, ListOauthAppsRequest, ListProjectsRequest, ListUsageRequest,
    OAuthTokenResponse, OauthApp, Project, ProjectInfo, ProjectInfoApp, ProjectWebhook,
    RateLimitRequest, RateLimitToken, SetWebhookRequest, UpdateAppOverrideRequest,
    UpdateOauthAppRequest, UpdateProjectLogoRequest, UpdateProjectRequest, UsageRecord, Webhook,
    WebhookWithOptionalSigningKey, WebhookWithSigningKey,
};
pub use error::{Error, IdentifierKind, Result, UrlField, ValidationField};
pub use execution::{
    ConfigurePropRequest, ConfigurePropResponse, DynamicProps, ErrorOrigin, ExecutionAttribution,
    ExecutionError, NetworkCall, Observation, ObservationError, ReloadPropsRequest,
    ReloadPropsResponse, RunActionRequest, RunActionResponse, StashReference,
};
pub use ids::{
    AccountId, AppCategoryId, AppOverrideId, AppSlug, ComponentKey, Cursor, DeployedTriggerId,
    ExternalUserId, OauthAppId, ProjectId, PropName, WebhookId, WorkflowEndpoint,
};
pub use pagination::{Page, PageInfo, PageRequest};
pub use props::{
    ConfigurableProp, ConfiguredPropValue, ConfiguredProps, PropKind, PropOption, PropOptionValue,
    TimerValue,
};
pub use registry::{
    App, AppAuthType, AppCategory, AppSortKey, Component, ComponentStash, ComponentType,
    GetComponentRequest, ListAppsRequest, ListComponentsRequest, ListRegistryRequest, ProxySupport,
    RegistryKind, ScopeProfile, ScopeProfileName, SortDirection, ToolAnnotations,
};
pub use secret::{ConnectToken, Environment, SecretJson, SecretString};
pub use shared::{Timestamp, UnknownFields};
pub use triggers::{
    DeleteTriggerRequest, DeployTriggerRequest, DeployTriggerResponse, DeployedComponent,
    DeployedTriggerQuery, EmittedEvent, Emitter, EmitterType, GetTriggerEventsResponse,
    GetTriggerResponse, GetTriggerWebhooksResponse, GetTriggerWorkflowsResponse,
    GetTriggersResponse, HttpInterface, ListDeployedTriggersRequest, ListTriggerEventsRequest,
    TimerInterface, TriggerWebhook, TriggerWebhooks, TriggerWorkflows, UpdateTriggerRequest,
    UpdateTriggerWebhooksRequest, UpdateTriggerWorkflowsRequest,
};
pub use wire::{
    ConnectUsageResponse, CreateAccountResponse, Data, ErrorMessage, ErrorResponse,
    GetAccountResponse, GetAccountsResponse, GetAppCategoryResponse, GetAppResponse,
    GetComponentResponse, GetComponentsResponse, GetUsersResponse, GetWebhookResponse,
    GetWebhookWithSigningKeyResponse, ListAccountsResponse, ListAppCategoriesResponse,
    ListAppOverridesResponse, ListAppsResponse, ListOauthAppsResponse, ListProjectsResponse,
    SetWebhookResponse,
};

pub(crate) use shared::string_enum;
