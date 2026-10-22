#![allow(unused_imports)]

use pipedream_connect::PipedreamClient;
use pipedream_connect::resources::proxy::ProxyRequest;
use pipedream_connect_types::{
    AccountId, AccountRef, AppCategoryId, AppOverrideId, AppSlug, ComponentKey,
    ConfigurePropRequest, ConnectToken, CreateAccountQuery, CreateAccountRequest,
    CreateAppOverrideRequest, CreateConnectTokenRequest, CreateOauthAppRequest,
    CreateProjectRequest, DeleteTriggerRequest, DeployedTriggerId, DeployedTriggerQuery,
    DownloadFileStashRequest, ExternalUserId, GetAccountRequest, GetComponentRequest,
    ListAccountsRequest, ListDeployedTriggersRequest, ListTriggerEventsRequest, ListUsageRequest,
    ListUserAccountsRequest, OauthAppId, PageRequest, ProjectId, ReloadPropsRequest,
    RunActionRequest, SecretString, SetWebhookRequest, UpdateAppOverrideRequest,
    UpdateOauthAppRequest, UpdateProjectLogoRequest, UpdateProjectRequest, UpdateTriggerRequest,
    UpdateTriggerWebhooksRequest, UpdateTriggerWorkflowsRequest, ValidateTokenRequest, WebhookId,
    WorkflowEndpoint,
};
use serde_json::json;

#[path = "admin.rs"]
mod admin;
#[path = "catalog.rs"]
mod catalog;

pub async fn invoke(client: &PipedreamClient, id: &str) -> bool {
    if catalog::invoke(client, id).await {
        return true;
    }
    admin::invoke(client, id).await
}

fn project_id() -> ProjectId {
    ProjectId::new("proj_Test123").unwrap()
}
fn account_id() -> AccountId {
    AccountId::new("apn_Account123").unwrap()
}
fn connect_token() -> ConnectToken {
    ConnectToken::new("ctok_00000000000000000000000000000000").unwrap()
}
fn account_request() -> CreateAccountRequest {
    serde_json::from_value(json!({"app_slug":"google_calendar","cfmap_json":"{}","connect_token":"ctok_00000000000000000000000000000000"})).unwrap()
}
fn configure_request() -> ConfigurePropRequest {
    serde_json::from_value(json!({"id":"google_calendar-list-events","external_user_id":"customer_1","prop_name":"CalendarID"})).unwrap()
}
fn reload_request() -> ReloadPropsRequest {
    serde_json::from_value(
        json!({"id":"google_calendar-list-events","external_user_id":"customer_1"}),
    )
    .unwrap()
}
fn run_request() -> RunActionRequest {
    serde_json::from_value(
        json!({"id":"google_calendar-list-events","external_user_id":"customer_1"}),
    )
    .unwrap()
}
fn deploy_request() -> pipedream_connect_types::DeployTriggerRequest {
    serde_json::from_value(
        json!({"id":"google_calendar-list-events","external_user_id":"customer_1"}),
    )
    .unwrap()
}
fn proxy_request(id: &str) -> ProxyRequest {
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
