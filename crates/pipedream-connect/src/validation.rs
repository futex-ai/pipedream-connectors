//! Request-owned validation required by JSON dispatch construction.

use pipedream_connect_types::{
    ConfigurePropRequest, CreateAccountRequest, CreateAppOverrideRequest,
    CreateConnectTokenRequest, CreateOauthAppRequest, CreateOauthTokenRequest,
    CreateProjectRequest, DeleteTriggerRequest, DeployTriggerRequest, DeployedTriggerQuery,
    DownloadFileStashRequest, GetAccountRequest, GetComponentRequest, ListAccountsRequest,
    ListAppOverridesRequest, ListAppsRequest, ListComponentsRequest, ListDeployedTriggersRequest,
    ListOauthAppsRequest, ListProjectsRequest, ListRegistryRequest, ListTriggerEventsRequest,
    ListUsageRequest, ListUserAccountsRequest, ListUsersRequest, PageRequest, RateLimitRequest,
    ReloadPropsRequest, RunActionRequest, SetWebhookRequest, UpdateAppOverrideRequest,
    UpdateOauthAppRequest, UpdateProjectLogoRequest, UpdateProjectRequest, UpdateTriggerRequest,
    UpdateTriggerWebhooksRequest, UpdateTriggerWorkflowsRequest, ValidateTokenRequest,
};

use crate::error::{Result, validate};

/// Validates a request model before its body is encoded or a bearer is acquired.
///
/// Each model owns its validation. Unvalidated serializable values cannot be sent.
pub trait Validate {
    /// Checks the model's documented input bounds.
    fn validate(&self) -> Result<()>;
}

impl Validate for CreateConnectTokenRequest {
    fn validate(&self) -> Result<()> {
        validate(CreateConnectTokenRequest::validate(self))
    }
}

impl Validate for PageRequest {
    fn validate(&self) -> Result<()> {
        validate(PageRequest::validate(self))
    }
}

macro_rules! paged_requests {
    ($($request:ty),+ $(,)?) => {$(
        impl Validate for $request {
            fn validate(&self) -> Result<()> {
                validate(self.page.validate())
            }
        }
    )+};
}

paged_requests!(
    ListAccountsRequest,
    ListAppOverridesRequest,
    ListAppsRequest,
    ListComponentsRequest,
    ListDeployedTriggersRequest,
    ListOauthAppsRequest,
    ListProjectsRequest,
    ListUsersRequest,
    ListRegistryRequest,
);

impl Validate for ListTriggerEventsRequest {
    fn validate(&self) -> Result<()> {
        validate(ListTriggerEventsRequest::new(
            self.external_user_id.clone(),
            self.limit,
        ))?;
        Ok(())
    }
}

/// These models have no additional bounds beyond their validated field types.
macro_rules! valid_by_construction {
    ($($request:ty),+ $(,)?) => {$(
        impl Validate for $request {
            fn validate(&self) -> Result<()> { Ok(()) }
        }
    )+};
}

valid_by_construction!(
    ConfigurePropRequest,
    CreateAccountRequest,
    CreateAppOverrideRequest,
    CreateOauthAppRequest,
    CreateOauthTokenRequest,
    CreateProjectRequest,
    DeleteTriggerRequest,
    DeployTriggerRequest,
    DeployedTriggerQuery,
    DownloadFileStashRequest,
    GetAccountRequest,
    GetComponentRequest,
    ListUsageRequest,
    ListUserAccountsRequest,
    RateLimitRequest,
    ReloadPropsRequest,
    RunActionRequest,
    SetWebhookRequest,
    UpdateAppOverrideRequest,
    UpdateOauthAppRequest,
    UpdateProjectLogoRequest,
    UpdateProjectRequest,
    UpdateTriggerRequest,
    UpdateTriggerWebhooksRequest,
    UpdateTriggerWorkflowsRequest,
    ValidateTokenRequest,
);
