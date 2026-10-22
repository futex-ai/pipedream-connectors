//! Pure request builders and identity checks for trigger lifecycle operations.

use crate::error::Result;
use crate::request::{ApiPath, ApiRequest, ResponseMode};
use http::Method;
use pipedream_connect_types::{
    DeleteTriggerRequest, DeployTriggerRequest, DeployedTriggerId, DeployedTriggerQuery, Emitter,
    GetTriggerResponse, GetWebhookWithSigningKeyResponse, ListDeployedTriggersRequest,
    ListTriggerEventsRequest, UpdateTriggerRequest, UpdateTriggerWebhooksRequest,
    UpdateTriggerWorkflowsRequest, WebhookId,
};
use serde_json::Value;

/// Builds a trigger deployment request.
pub(super) fn deploy(request: &DeployTriggerRequest) -> Result<ApiRequest> {
    ApiRequest::new(Method::POST, ApiPath::project(&["triggers", "deploy"])).with_json(request)
}

/// Builds a deployed-trigger list request.
pub(super) fn list(request: &ListDeployedTriggersRequest) -> Result<ApiRequest> {
    ApiRequest::new(Method::GET, ApiPath::project(&["deployed-triggers"])).with_query(request)
}

/// Builds a deployed-trigger retrieval request.
pub(super) fn get(id: &DeployedTriggerId, request: &DeployedTriggerQuery) -> Result<ApiRequest> {
    path(Method::GET, id, &[]).with_query(request)
}

/// Builds a deployed-trigger update request.
pub(super) fn update(
    id: &DeployedTriggerId,
    query: &DeployedTriggerQuery,
    request: &UpdateTriggerRequest,
) -> Result<ApiRequest> {
    path(Method::PUT, id, &[])
        .with_query(query)?
        .with_json(request)
}

/// Builds a deployed-trigger delete request.
pub(super) fn delete(id: &DeployedTriggerId, request: &DeleteTriggerRequest) -> Result<ApiRequest> {
    Ok(path(Method::DELETE, id, &[])
        .with_query(request)?
        .with_response_mode(ResponseMode::Delete {
            missing_ok: request.missing_ok,
        }))
}

/// Builds a deployed-trigger event-history request.
pub(super) fn events(
    id: &DeployedTriggerId,
    request: &ListTriggerEventsRequest,
) -> Result<ApiRequest> {
    path(Method::GET, id, &["events"]).with_query(request)
}

/// Builds a deployed-trigger workflow retrieval request.
pub(super) fn workflows(
    id: &DeployedTriggerId,
    request: &DeployedTriggerQuery,
) -> Result<ApiRequest> {
    path(Method::GET, id, &["pipelines"]).with_query(request)
}

/// Builds a deployed-trigger workflow replacement request.
pub(super) fn update_workflows(
    id: &DeployedTriggerId,
    query: &DeployedTriggerQuery,
    request: &UpdateTriggerWorkflowsRequest,
) -> Result<ApiRequest> {
    path(Method::PUT, id, &["pipelines"])
        .with_query(query)?
        .with_json(request)
}

/// Builds a deployed-trigger webhook retrieval request.
pub(super) fn webhooks(
    id: &DeployedTriggerId,
    request: &DeployedTriggerQuery,
) -> Result<ApiRequest> {
    path(Method::GET, id, &["webhooks"]).with_query(request)
}

/// Builds a deployed-trigger webhook replacement request.
pub(super) fn update_webhooks(
    id: &DeployedTriggerId,
    query: &DeployedTriggerQuery,
    request: &UpdateTriggerWebhooksRequest,
) -> Result<ApiRequest> {
    path(Method::PUT, id, &["webhooks"])
        .with_query(query)?
        .with_json(request)
}

/// Builds a single deployed-trigger webhook retrieval request.
pub(super) fn webhook(
    id: &DeployedTriggerId,
    webhook_id: &WebhookId,
    request: &DeployedTriggerQuery,
) -> Result<ApiRequest> {
    path(Method::GET, id, &["webhooks", webhook_id.as_str()]).with_query(request)
}

/// Builds a signing-key regeneration request for one deployed-trigger webhook.
pub(super) fn regenerate_webhook_signing_key(
    id: &DeployedTriggerId,
    webhook_id: &WebhookId,
    request: &DeployedTriggerQuery,
) -> Result<ApiRequest> {
    path(
        Method::POST,
        id,
        &["webhooks", webhook_id.as_str(), "regenerate_signing_key"],
    )
    .with_query(request)
}

fn path(method: Method, id: &DeployedTriggerId, suffix: &[&str]) -> ApiRequest {
    let mut segments = vec!["deployed-triggers".to_owned(), id.as_str().to_owned()];
    segments.extend(suffix.iter().map(|segment| (*segment).to_owned()));
    ApiRequest::new(method, ApiPath::Project(segments))
}

/// Checks a deployed-trigger response against its requested identity.
pub(super) fn check_trigger(
    response: GetTriggerResponse,
    requested: &DeployedTriggerId,
) -> Result<GetTriggerResponse> {
    let mismatch = match &response.data {
        Emitter::DeployedComponent(value) => value.id.as_str() != requested.as_str(),
        Emitter::HttpInterface(value) => value.id != requested.as_str(),
        Emitter::TimerInterface(value) => value.id != requested.as_str(),
        Emitter::Other(value) => {
            value.get("id").and_then(Value::as_str) != Some(requested.as_str())
        }
    };
    super::operation::ensure_identity(response, !mismatch)
}

/// Checks a webhook response against its requested destination.
pub(super) fn check_webhook(
    response: GetWebhookWithSigningKeyResponse,
    requested: &WebhookId,
) -> Result<GetWebhookWithSigningKeyResponse> {
    let matches = &response.data.id == requested;
    super::operation::ensure_identity(response, matches)
}
