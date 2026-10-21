//! Pure registry paths and identity checks shared by resource handles.

use http::Method;
use pipedream_connect_types::{ComponentKey, GetComponentRequest, GetComponentResponse};

use crate::error::{Error, Result, WireFailure};
use crate::request::{ApiPath, ApiRequest};

/// The registry resource route, independent of a component's provider-defined kind.
#[derive(Clone, Copy)]
pub(super) enum CatalogGroup {
    Components,
    Actions,
    Triggers,
}

impl CatalogGroup {
    pub(super) fn segment(self) -> &'static str {
        match self {
            Self::Components => "components",
            Self::Actions => "actions",
            Self::Triggers => "triggers",
        }
    }
}

pub(super) fn get_request(
    group: CatalogGroup,
    key: &ComponentKey,
    request: GetComponentRequest,
) -> ApiRequest {
    let operation = ApiRequest::new(
        Method::GET,
        ApiPath::project(&[group.segment(), key.as_str()]),
    );
    match request.version {
        Some(version) => operation.query_pair("version", version),
        None => operation,
    }
}

pub(super) fn check_component(
    response: GetComponentResponse,
    key: &ComponentKey,
) -> Result<GetComponentResponse> {
    if &response.data.key != key {
        return Err(Error::internal(WireFailure::IdentityMismatch));
    }
    Ok(response)
}
