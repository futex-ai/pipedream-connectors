//! Pure request builders and response identity checks for administration calls.

use http::header::CONTENT_TYPE;
use http::{HeaderMap, HeaderValue, Method};
use pipedream_connect_types::{
    AppOverride, AppOverrideId, CreateAppOverrideRequest, CreateOauthAppRequest,
    CreateProjectRequest, DownloadFileStashRequest, ExternalUserId, ListAppOverridesRequest,
    ListOauthAppsRequest, ListProjectsRequest, ListUsageRequest, OauthApp, OauthAppId, Project,
    ProjectId, RateLimitRequest, SetWebhookRequest, UpdateAppOverrideRequest,
    UpdateOauthAppRequest, UpdateProjectLogoRequest, UpdateProjectRequest, WorkflowRequest,
};

use crate::error::{Error, InvalidRequest, Result};
use crate::request::{ApiPath, ApiRequest, ResponseMode};

/// Builds the project list request.
pub(super) fn list_projects(request: &ListProjectsRequest) -> Result<ApiRequest> {
    ApiRequest::new(Method::GET, ApiPath::connect(&["projects"])).with_query(request)
}

/// Builds the project creation request.
pub(super) fn create_project(request: &CreateProjectRequest) -> Result<ApiRequest> {
    ApiRequest::new(Method::POST, ApiPath::connect(&["projects"])).with_json(request)
}

/// Builds a project retrieval request.
pub(super) fn get_project(project_id: &ProjectId) -> ApiRequest {
    ApiRequest::new(
        Method::GET,
        ApiPath::connect(&["projects", project_id.as_str()]),
    )
}

/// Checks a project response against its requested identifier.
pub(super) fn check_project(project: Project, requested: &ProjectId) -> Result<Project> {
    let matches = &project.id == requested;
    super::operation::ensure_identity(project, matches)
}

/// Builds a project update request.
pub(super) fn update_project(
    project_id: &ProjectId,
    request: &UpdateProjectRequest,
) -> Result<ApiRequest> {
    ApiRequest::new(
        Method::PATCH,
        ApiPath::connect(&["projects", project_id.as_str()]),
    )
    .with_json(request)
}

/// Builds a project deletion request.
pub(super) fn delete_project(project_id: &ProjectId) -> ApiRequest {
    ApiRequest::new(
        Method::DELETE,
        ApiPath::connect(&["projects", project_id.as_str()]),
    )
}

/// Builds a project logo update request.
pub(super) fn update_project_logo(
    project_id: &ProjectId,
    request: &UpdateProjectLogoRequest,
) -> Result<ApiRequest> {
    ApiRequest::new(
        Method::POST,
        ApiPath::connect(&["projects", project_id.as_str(), "logo"]),
    )
    .with_json(request)
}

/// Builds the enabled-apps information request for the configured project.
pub(super) fn project_info() -> ApiRequest {
    ApiRequest::new(Method::GET, ApiPath::project(&["projects", "info"]))
}

/// Builds a custom OAuth app list request.
pub(super) fn list_oauth_apps(request: &ListOauthAppsRequest) -> Result<ApiRequest> {
    ApiRequest::new(Method::GET, ApiPath::project(&["oauth_apps"])).with_query(request)
}

/// Builds a custom OAuth app creation request.
pub(super) fn create_oauth_app(request: &CreateOauthAppRequest) -> Result<ApiRequest> {
    ApiRequest::new(Method::POST, ApiPath::project(&["oauth_apps"])).with_json(request)
}

/// Builds a custom OAuth app retrieval request.
pub(super) fn get_oauth_app(id: &OauthAppId) -> ApiRequest {
    ApiRequest::new(Method::GET, ApiPath::project(&["oauth_apps", id.as_str()]))
}

/// Checks a custom OAuth app response against its requested identifier.
pub(super) fn check_oauth_app(app: OauthApp, requested: &OauthAppId) -> Result<OauthApp> {
    let matches = &app.id == requested;
    super::operation::ensure_identity(app, matches)
}

/// Builds a custom OAuth app update request.
pub(super) fn update_oauth_app(
    id: &OauthAppId,
    request: &UpdateOauthAppRequest,
) -> Result<ApiRequest> {
    ApiRequest::new(Method::PUT, ApiPath::project(&["oauth_apps", id.as_str()])).with_json(request)
}

/// Builds a custom OAuth app deletion request.
pub(super) fn delete_oauth_app(id: &OauthAppId) -> ApiRequest {
    ApiRequest::new(
        Method::DELETE,
        ApiPath::project(&["oauth_apps", id.as_str()]),
    )
}

/// Builds an app override list request.
pub(super) fn list_app_overrides(request: &ListAppOverridesRequest) -> Result<ApiRequest> {
    ApiRequest::new(Method::GET, ApiPath::project(&["app_overrides"])).with_query(request)
}

/// Builds an app override creation request.
pub(super) fn create_app_override(request: &CreateAppOverrideRequest) -> Result<ApiRequest> {
    ApiRequest::new(Method::POST, ApiPath::project(&["app_overrides"])).with_json(request)
}

/// Builds an app override retrieval request.
pub(super) fn get_app_override(id: &AppOverrideId) -> ApiRequest {
    ApiRequest::new(
        Method::GET,
        ApiPath::project(&["app_overrides", id.as_str()]),
    )
}

/// Checks an app override response against its requested identifier.
pub(super) fn check_app_override(
    value: AppOverride,
    requested: &AppOverrideId,
) -> Result<AppOverride> {
    let matches = &value.id == requested;
    super::operation::ensure_identity(value, matches)
}

/// Builds an app override update request.
pub(super) fn update_app_override(
    id: &AppOverrideId,
    request: &UpdateAppOverrideRequest,
) -> Result<ApiRequest> {
    ApiRequest::new(
        Method::PUT,
        ApiPath::project(&["app_overrides", id.as_str()]),
    )
    .with_json(request)
}

/// Builds an app override deletion request.
pub(super) fn delete_app_override(id: &AppOverrideId) -> ApiRequest {
    ApiRequest::new(
        Method::DELETE,
        ApiPath::project(&["app_overrides", id.as_str()]),
    )
}

/// Builds a project environment webhook retrieval request.
pub(super) fn get_project_webhook() -> ApiRequest {
    ApiRequest::new(Method::GET, ApiPath::project(&["webhook"]))
}

/// Builds a project environment webhook update request.
pub(super) fn set_project_webhook(request: &SetWebhookRequest) -> Result<ApiRequest> {
    ApiRequest::new(Method::PUT, ApiPath::project(&["webhook"])).with_json(request)
}

/// Builds a project environment webhook deletion request.
pub(super) fn delete_project_webhook() -> ApiRequest {
    ApiRequest::new(Method::DELETE, ApiPath::project(&["webhook"]))
}

/// Builds a project environment webhook signing-key regeneration request.
pub(super) fn regenerate_project_webhook() -> ApiRequest {
    ApiRequest::new(
        Method::POST,
        ApiPath::project(&["webhook", "regenerate_signing_key"]),
    )
}

/// Builds a usage query request.
pub(super) fn list_usage(request: &ListUsageRequest) -> Result<ApiRequest> {
    ApiRequest::new(Method::GET, ApiPath::connect(&["usage"])).with_query(request)
}

/// Builds a custom rate-limit token request.
pub(super) fn create_rate_limit(request: &RateLimitRequest) -> Result<ApiRequest> {
    ApiRequest::new(Method::POST, ApiPath::connect(&["rate_limits"])).with_json(request)
}

/// Builds a file-stash request that accepts either bytes or a redirect.
pub(super) fn download_file_stash(request: &DownloadFileStashRequest) -> Result<ApiRequest> {
    Ok(
        ApiRequest::new(Method::GET, ApiPath::project(&["file_stash", "download"]))
            .with_query(request)?
            .with_response_mode(ResponseMode::Redirect),
    )
}

/// Builds a workflow invocation against an absolute endpoint URL.
pub(super) fn invoke_workflow(
    endpoint: &str,
    method: Method,
    external_user_id: &ExternalUserId,
    request: WorkflowRequest,
) -> Result<ApiRequest> {
    let mut headers = HeaderMap::new();
    let value = match HeaderValue::from_str(external_user_id.as_str()) {
        Ok(value) => value,
        Err(_) => return Err(Error::InvalidRequest(InvalidRequest::Header)),
    };
    headers.insert("x-pd-external-user-id", value);
    if let Some(content_type) = request.content_type {
        let value = match HeaderValue::from_str(&content_type) {
            Ok(value) => value,
            Err(_) => return Err(Error::InvalidRequest(InvalidRequest::Header)),
        };
        headers.insert(CONTENT_TYPE, value);
    }
    let operation = ApiRequest::new(method, ApiPath::workflow_endpoint(endpoint)?)
        .with_headers(headers)
        .with_response_mode(ResponseMode::WorkflowPassThrough);
    Ok(match request.body {
        Some(body) => operation.with_bytes(body),
        None => operation,
    })
}
