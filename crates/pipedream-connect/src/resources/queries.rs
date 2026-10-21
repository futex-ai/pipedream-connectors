//! Typed list requests encoded as query pairs before shared dispatch.

use http::Method;
use pipedream_connect_types::{ListAppsRequest, ListComponentsRequest, ListRegistryRequest};
use serde::Serialize;

use crate::error::Result;
use crate::json::{decode, encode};
use crate::request::{ApiPath, ApiRequest};

use super::catalog::CatalogGroup;

pub(super) fn apps(request: ListAppsRequest) -> Result<ApiRequest> {
    let mut operation =
        ApiRequest::new(Method::GET, ApiPath::connect(&["apps"])).with_page(&request.page)?;
    operation = optional(operation, "q", request.query);
    operation = string_enum(operation, "sort_key", request.sort_key)?;
    operation = string_enum(operation, "sort_direction", request.sort_direction)?;
    for category in request.category_ids {
        operation = operation.query_pair("category_ids", category.as_str());
    }
    for (name, value) in [
        ("has_components", request.has_components),
        ("has_actions", request.has_actions),
        ("has_triggers", request.has_triggers),
    ] {
        if let Some(value) = value {
            operation = operation.query_pair(name, value.to_string());
        }
    }
    Ok(operation)
}

pub(super) fn components(request: ListComponentsRequest) -> Result<ApiRequest> {
    let operation = registry(
        CatalogGroup::Components,
        ListRegistryRequest {
            page: request.page,
            query: request.query,
            app: request.app,
            registry: request.registry,
        },
    )?;
    string_enum(operation, "component_type", request.component_type)
}

pub(super) fn registry(group: CatalogGroup, request: ListRegistryRequest) -> Result<ApiRequest> {
    let mut operation = ApiRequest::new(Method::GET, ApiPath::project(&[group.segment()]))
        .with_page(&request.page)?;
    operation = optional(operation, "q", request.query);
    operation = optional(
        operation,
        "app",
        request.app.map(|app| app.as_str().to_owned()),
    );
    string_enum(operation, "registry", request.registry)
}

fn optional(operation: ApiRequest, name: &str, value: Option<String>) -> ApiRequest {
    match value {
        Some(value) => operation.query_pair(name, value),
        None => operation,
    }
}

/// Uses the model's serde wire value, including enum values added by the provider.
fn string_enum<T: Serialize>(
    operation: ApiRequest,
    name: &str,
    value: Option<T>,
) -> Result<ApiRequest> {
    match value {
        Some(value) => Ok(operation.query_pair(name, decode::<String>(&encode(&value)?)?)),
        None => Ok(operation),
    }
}
