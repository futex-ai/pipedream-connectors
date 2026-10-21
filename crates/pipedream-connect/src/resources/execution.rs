//! Pure execution requests with model-owned validation and explicit 202 handling.

use http::Method;
use pipedream_connect_types::{ConfigurePropRequest, ReloadPropsRequest, RunActionRequest};

use crate::error::Result;
use crate::request::{ApiPath, ApiRequest, ResponseMode};

use super::catalog::CatalogGroup;

pub(super) fn configure(group: CatalogGroup, request: &ConfigurePropRequest) -> Result<ApiRequest> {
    Ok(ApiRequest::new(
        Method::POST,
        ApiPath::project(&[group.segment(), "configure"]),
    )
    .with_json(request)?
    .with_response_mode(ResponseMode::Execution {
        nonblocking: request.blocking == Some(false),
    }))
}

pub(super) fn reload(group: CatalogGroup, request: &ReloadPropsRequest) -> Result<ApiRequest> {
    Ok(
        ApiRequest::new(Method::POST, ApiPath::project(&[group.segment(), "props"]))
            .with_json(request)?
            .with_response_mode(ResponseMode::Execution {
                nonblocking: request.blocking == Some(false),
            }),
    )
}

pub(super) fn run(request: &RunActionRequest) -> Result<ApiRequest> {
    Ok(
        ApiRequest::new(Method::POST, ApiPath::project(&["actions", "run"]))
            .with_json(request)?
            .with_response_mode(ResponseMode::Execution {
                nonblocking: request.blocking == Some(false),
            }),
    )
}

#[cfg(test)]
#[path = "_tests_/execution_tests.rs"]
mod execution_tests;
