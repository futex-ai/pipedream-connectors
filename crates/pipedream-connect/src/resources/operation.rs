//! Shared resource request builders, dispatch, decoding, and identity checks.

use std::sync::Arc;

use http::Method;
use pipedream_connect_types::{
    AppCategoryId, AppSlug, ComponentKey, ConfigurePropRequest, GetAppCategoryResponse,
    GetAppResponse, GetComponentRequest, GetComponentResponse, ListAppsRequest,
    ListComponentsRequest, ListRegistryRequest, ReloadPropsRequest, RunActionRequest,
};
use serde::de::DeserializeOwned;

use crate::client::RequestDispatcher;
use crate::error::{Error, Result, WireFailure};
use crate::request::{ApiPath, ApiRequest, ResponseMode};
use crate::transport::HttpResponse;

/// A resource catalog route.
#[derive(Clone, Copy)]
pub(super) enum CatalogGroup {
    /// Generic registry components.
    Components,
    /// Registry actions.
    Actions,
    /// Registry triggers.
    Triggers,
}

impl CatalogGroup {
    /// Returns the route segment for this catalog group.
    pub(super) fn segment(self) -> &'static str {
        match self {
            Self::Components => "components",
            Self::Actions => "actions",
            Self::Triggers => "triggers",
        }
    }
}

/// Dispatches an operation and decodes its JSON response.
pub(super) async fn json<T: DeserializeOwned>(
    dispatcher: &Arc<dyn RequestDispatcher + Send + Sync>,
    request: Result<ApiRequest>,
) -> Result<T> {
    dispatcher.dispatch(request?).await?.json()
}

/// Dispatches an operation while retaining its status, headers, and body.
pub(super) async fn dispatch(
    dispatcher: &Arc<dyn RequestDispatcher + Send + Sync>,
    request: Result<ApiRequest>,
) -> Result<HttpResponse> {
    dispatcher.dispatch(request?).await
}

/// Dispatches an operation that has no response body contract.
pub(super) async fn no_content(
    dispatcher: &Arc<dyn RequestDispatcher + Send + Sync>,
    request: Result<ApiRequest>,
) -> Result<()> {
    dispatch(dispatcher, request).await.map(|_| ())
}

/// Returns a response after checking a read identity predicate.
pub(super) fn ensure_identity<T>(value: T, matches: bool) -> Result<T> {
    if matches {
        Ok(value)
    } else {
        Err(Error::internal(WireFailure::IdentityMismatch))
    }
}

/// Builds the app list request.
pub(super) fn list_apps(request: &ListAppsRequest) -> Result<ApiRequest> {
    ApiRequest::new(Method::GET, ApiPath::connect(&["apps"])).with_query(request)
}

/// Builds the app retrieval request.
pub(super) fn get_app(slug: &AppSlug) -> ApiRequest {
    ApiRequest::new(Method::GET, ApiPath::connect(&["apps", slug.as_str()]))
}

/// Checks an app response against either its requested slug or ID.
pub(super) fn check_app(response: GetAppResponse, requested: &AppSlug) -> Result<GetAppResponse> {
    let matches_slug = response.data.name_slug.as_str() == requested.as_str();
    let matches_id = response.data.id.as_deref() == Some(requested.as_str());
    ensure_identity(response, matches_slug || matches_id)
}

/// Builds the category list request.
pub(super) fn list_categories() -> ApiRequest {
    ApiRequest::new(Method::GET, ApiPath::connect(&["app_categories"]))
}

/// Builds the category retrieval request.
pub(super) fn get_category(id: &AppCategoryId) -> ApiRequest {
    ApiRequest::new(
        Method::GET,
        ApiPath::connect(&["app_categories", id.as_str()]),
    )
}

/// Checks a category response against its requested identifier.
pub(super) fn check_category(
    response: GetAppCategoryResponse,
    requested: &AppCategoryId,
) -> Result<GetAppCategoryResponse> {
    let matches = response.id.as_ref() == Some(requested);
    ensure_identity(response, matches)
}

/// Builds a component list request.
pub(super) fn list_components(request: &ListComponentsRequest) -> Result<ApiRequest> {
    ApiRequest::new(
        Method::GET,
        ApiPath::project(&[CatalogGroup::Components.segment()]),
    )
    .with_query(request)
}

/// Builds an action or trigger list request.
pub(super) fn list_registry(
    group: CatalogGroup,
    request: &ListRegistryRequest,
) -> Result<ApiRequest> {
    ApiRequest::new(Method::GET, ApiPath::project(&[group.segment()])).with_query(request)
}

/// Builds a versioned catalog retrieval request.
pub(super) fn get_component(
    group: CatalogGroup,
    key: &ComponentKey,
    request: &GetComponentRequest,
) -> Result<ApiRequest> {
    ApiRequest::new(
        Method::GET,
        ApiPath::project(&[group.segment(), key.as_str()]),
    )
    .with_query(request)
}

/// Checks a catalog response against its requested component key.
pub(super) fn check_component(
    response: GetComponentResponse,
    requested: &ComponentKey,
) -> Result<GetComponentResponse> {
    let matches = &response.data.key == requested;
    ensure_identity(response, matches)
}

/// Builds a property configuration request.
pub(super) fn configure(group: CatalogGroup, request: &ConfigurePropRequest) -> Result<ApiRequest> {
    Ok(ApiRequest::new(
        Method::POST,
        ApiPath::project(&[group.segment(), "configure"]),
    )
    .with_json(request)?
    .with_response_mode(ResponseMode::Execution))
}

/// Builds a dynamic-property reload request.
pub(super) fn reload(group: CatalogGroup, request: &ReloadPropsRequest) -> Result<ApiRequest> {
    Ok(
        ApiRequest::new(Method::POST, ApiPath::project(&[group.segment(), "props"]))
            .with_json(request)?
            .with_response_mode(ResponseMode::Execution),
    )
}

/// Builds an action run request. Every 202 response is accepted.
pub(super) fn run(request: &RunActionRequest) -> Result<ApiRequest> {
    Ok(
        ApiRequest::new(Method::POST, ApiPath::project(&["actions", "run"]))
            .with_json(request)?
            .with_response_mode(ResponseMode::Execution),
    )
}

#[cfg(test)]
#[path = "_tests_/execution_tests.rs"]
mod execution_tests;
