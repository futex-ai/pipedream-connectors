//! Shared inputs and dispatch for OpenAPI coverage cases.

use pipedream_connect::PipedreamClient;
use pipedream_connect::resources::proxy::ProxyRequest;
use pipedream_connect_types::{
    AccountId, AccountRef, ConfigurePropRequest, ConnectToken, CreateAccountRequest,
    ExternalUserId, ProjectId, ReloadPropsRequest, RunActionRequest,
};
use serde_json::json;

use super::{admin, catalog};

pub async fn invoke(client: &PipedreamClient, id: &str) -> bool {
    if catalog::invoke(client, id).await {
        return true;
    }
    admin::invoke(client, id).await
}

pub(super) fn project_id() -> ProjectId {
    ProjectId::new("proj_Test123").unwrap()
}
pub(super) fn account_id() -> AccountId {
    AccountId::new("apn_Account123").unwrap()
}
pub(super) fn connect_token() -> ConnectToken {
    ConnectToken::new("ctok_00000000000000000000000000000000").unwrap()
}
pub(super) fn account_request() -> CreateAccountRequest {
    serde_json::from_value(json!({"app_slug":"google_calendar","cfmap_json":"{}","connect_token":"ctok_00000000000000000000000000000000"})).unwrap()
}
pub(super) fn configure_request() -> ConfigurePropRequest {
    serde_json::from_value(json!({"id":"google_calendar-list-events","external_user_id":"customer_1","prop_name":"CalendarID"})).unwrap()
}
pub(super) fn reload_request() -> ReloadPropsRequest {
    serde_json::from_value(
        json!({"id":"google_calendar-list-events","external_user_id":"customer_1"}),
    )
    .unwrap()
}
pub(super) fn run_request() -> RunActionRequest {
    serde_json::from_value(
        json!({"id":"google_calendar-list-events","external_user_id":"customer_1"}),
    )
    .unwrap()
}
pub(super) fn deploy_request() -> pipedream_connect_types::DeployTriggerRequest {
    serde_json::from_value(
        json!({"id":"google_calendar-list-events","external_user_id":"customer_1"}),
    )
    .unwrap()
}
pub(super) fn proxy_request(id: &str) -> ProxyRequest {
    let account = AccountRef::new(ExternalUserId::new("customer_1").unwrap(), account_id());
    let target = "/v1/events";
    match id {
        "retrieveProxy" => ProxyRequest::get(account, target),
        "postProxy" => ProxyRequest::post(account, target).with_body(b"body".to_vec()),
        "putProxy" => ProxyRequest::put(account, target).with_body(b"body".to_vec()),
        "deleteProxy" => ProxyRequest::delete(account, target),
        "patchProxy" => ProxyRequest::patch(account, target).with_body(b"body".to_vec()),
        _ => unreachable!(),
    }
}
