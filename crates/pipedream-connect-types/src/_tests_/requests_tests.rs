//! Golden wire bodies and queries for every request model.

use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use crate::*;

type Check = fn(Value);

fn golden<T: DeserializeOwned + Serialize>(expected: Value) {
    let model: T = serde_json::from_value(expected.clone()).unwrap();
    assert_eq!(serde_json::to_value(model).unwrap(), expected);
}

#[test]
fn every_request_encodes_its_documented_body_or_query() {
    let props = json!({"account":{"authProvisionId":"apn_sample"},"channel":"C1","large":9007199254740993u64});
    let cases: Vec<(&str, Check, Value)> = vec![
        (
            "PageRequest",
            golden::<PageRequest>,
            json!({"limit":1,"after":"cursor"}),
        ),
        (
            "CreateAccountRequest",
            golden::<CreateAccountRequest>,
            json!({"app_slug":"github","cfmap_json":"{\"api_key\":\"value\"}","connect_token":"ctok_0123456789abcdef0123456789abcdef","name":"Account","account_id":"apn_sample"}),
        ),
        (
            "CreateAccountQuery",
            golden::<CreateAccountQuery>,
            json!({"external_user_id":"user","oauth_app_id":"oa_sample"}),
        ),
        (
            "ListAccountsRequest",
            golden::<ListAccountsRequest>,
            json!({"limit":1,"after":"cursor","external_user_id":"user","app":"github","oauth_app_id":"oa_sample","include_credentials":true}),
        ),
        (
            "ListUsersRequest",
            golden::<ListUsersRequest>,
            json!({"limit":1,"q":"search"}),
        ),
        (
            "ListUserAccountsRequest",
            golden::<ListUserAccountsRequest>,
            json!({"app":"github","include_credentials":false}),
        ),
        (
            "GetAccountRequest",
            golden::<GetAccountRequest>,
            json!({"include_credentials":true}),
        ),
        (
            "ValidateTokenRequest",
            golden::<ValidateTokenRequest>,
            json!({"app_id":"app_sample","account_id":"apn_sample","oauth_app_id":"oa_sample"}),
        ),
        (
            "CreateConnectTokenRequest",
            golden::<CreateConnectTokenRequest>,
            json!({"project_id":"proj_sample","external_user_id":"user","expires_in":1,"allowed_origins":["https://example.com"],"success_redirect_uri":"https://example.com/success","error_redirect_uri":"https://example.com/error","webhook_uri":"https://example.com/hook","scope":"connect:*","allow_progressive_scopes":true}),
        ),
        (
            "ListAppsRequest",
            golden::<ListAppsRequest>,
            json!({"limit":1,"q":"search","sort_key":"name_slug","sort_direction":"desc","category_ids":["appcat_sample"],"has_components":true,"has_actions":false,"has_triggers":true}),
        ),
        (
            "ListComponentsRequest",
            golden::<ListComponentsRequest>,
            json!({"limit":1,"q":"search","app":"github","registry":"all","component_type":"trigger"}),
        ),
        (
            "GetComponentRequest",
            golden::<GetComponentRequest>,
            json!({"version":"1.2.3"}),
        ),
        (
            "ConfigurePropRequest",
            golden::<ConfigurePropRequest>,
            json!({"id":"github-list","version":"1.2.3","external_user_id":"user","prop_name":"channel","blocking":true,"configured_props":props,"dynamic_props_id":"dynamic","page":2,"prev_context":{"cursor":1},"query":"search"}),
        ),
        (
            "ReloadPropsRequest",
            golden::<ReloadPropsRequest>,
            json!({"id":"github-list","version":"1.2.3","external_user_id":"user","blocking":false,"configured_props":props,"dynamic_props_id":"dynamic"}),
        ),
        (
            "RunActionRequest",
            golden::<RunActionRequest>,
            json!({"id":"github-list","version":"1.2.3","external_user_id":"user","configured_props":props,"dynamic_props_id":"dynamic","stash_id":"stash_sample"}),
        ),
        (
            "DeployTriggerRequest",
            golden::<DeployTriggerRequest>,
            json!({"id":"github-trigger","version":"1.2.3","external_user_id":"user","configured_props":props,"dynamic_props_id":"dynamic","workflow_id":"p_sample","webhook_url":"https://example.com/hook","emit_on_deploy":false}),
        ),
        (
            "UpdateTriggerRequest",
            golden::<UpdateTriggerRequest>,
            json!({"active":true,"configured_props":props,"name":"Trigger","emit_on_deploy":false}),
        ),
        (
            "ListDeployedTriggersRequest",
            golden::<ListDeployedTriggersRequest>,
            json!({"external_user_id":"user","limit":1,"emitter_type":"source"}),
        ),
        (
            "ListTriggerEventsRequest",
            golden::<ListTriggerEventsRequest>,
            json!({"external_user_id":"user","n":100}),
        ),
        (
            "DeleteTriggerRequest",
            golden::<DeleteTriggerRequest>,
            json!({"external_user_id":"user","ignore_hook_errors":true}),
        ),
        (
            "DeployedTriggerQuery",
            golden::<DeployedTriggerQuery>,
            json!({"external_user_id":"user"}),
        ),
        (
            "UpdateTriggerWebhooksRequest",
            golden::<UpdateTriggerWebhooksRequest>,
            json!({"webhook_urls":["https://example.com/hook"]}),
        ),
        (
            "UpdateTriggerWorkflowsRequest",
            golden::<UpdateTriggerWorkflowsRequest>,
            json!({"workflow_ids":["p_sample"]}),
        ),
        (
            "CreateProjectRequest",
            golden::<CreateProjectRequest>,
            json!({"name":"Project","app_name":"App","support_email":"user@example.com","connect_require_key_auth_test":true}),
        ),
        (
            "UpdateProjectRequest",
            golden::<UpdateProjectRequest>,
            json!({"name":"Project","app_name":"App","support_email":"user@example.com","connect_require_key_auth_test":false}),
        ),
        (
            "UpdateProjectLogoRequest",
            golden::<UpdateProjectLogoRequest>,
            json!({"logo":"data:image/png;base64,AA=="}),
        ),
        (
            "CreateOauthAppRequest",
            golden::<CreateOauthAppRequest>,
            json!({"app":"github","client_id":"client","client_secret":"secret","name":"App","description":"Example","scopes":["read"],"additional_scopes":["write"]}),
        ),
        (
            "UpdateOauthAppRequest",
            golden::<UpdateOauthAppRequest>,
            json!({"client_id":"client","client_secret":"secret","name":"App","description":"Example","scopes":["read"],"additional_scopes":["write"]}),
        ),
        (
            "CreateAppOverrideRequest",
            golden::<CreateAppOverrideRequest>,
            json!({"app":"github","name":"Override","oauth_app_id":"oa_sample","cfmap":{"domain":"example.com"}}),
        ),
        (
            "UpdateAppOverrideRequest",
            golden::<UpdateAppOverrideRequest>,
            json!({"name":"Override","oauth_app_id":null,"cfmap":{"domain":"example.com"}}),
        ),
        (
            "ListProjectsRequest",
            golden::<ListProjectsRequest>,
            json!({"limit":1,"q":"search"}),
        ),
        (
            "ListOauthAppsRequest",
            golden::<ListOauthAppsRequest>,
            json!({"limit":1,"app":"github","q":"search"}),
        ),
        (
            "ListAppOverridesRequest",
            golden::<ListAppOverridesRequest>,
            json!({"limit":1,"app":"github"}),
        ),
        (
            "ListUsageRequest",
            golden::<ListUsageRequest>,
            json!({"start_ts":1000,"end_ts":2000}),
        ),
        (
            "DownloadFileStashRequest",
            golden::<DownloadFileStashRequest>,
            json!({"s3_key":"1day/a/b/c/u/file.txt"}),
        ),
        (
            "SetWebhookRequest",
            golden::<SetWebhookRequest>,
            json!({"url":"https://example.com/hook"}),
        ),
        (
            "RateLimitRequest",
            golden::<RateLimitRequest>,
            json!({"window_size_seconds":10,"requests_per_window":1000}),
        ),
        (
            "CreateOauthTokenRequest",
            golden::<CreateOauthTokenRequest>,
            json!({"grant_type":"client_credentials","client_id":"client","client_secret":"secret","scope":"connect:*"}),
        ),
    ];
    for (name, check, sample) in cases {
        assert!(std::panic::catch_unwind(|| check(sample)).is_ok(), "{name}");
    }
}

#[test]
fn stash_references_encode_ids_new_and_boolean_values() {
    for stash in [
        json!("stash_sample"),
        json!("NEW"),
        json!(true),
        json!(false),
    ] {
        let sample = json!({"id":"github-list","external_user_id":"user","stash_id":stash});
        golden::<RunActionRequest>(sample);
    }
}

#[test]
fn trigger_event_limits_use_typed_errors_and_external_user_scope() {
    for limit in [0, 1, 100, 101] {
        let value = ListTriggerEventsRequest::new(ExternalUserId::new("user").unwrap(), limit);
        assert_eq!(value.is_ok(), (1..=100).contains(&limit));
    }
}

#[test]
fn trigger_event_limit_is_checked_during_deserialization() {
    for limit in [0, 1, 100, 101] {
        let decoded = serde_json::from_value::<ListTriggerEventsRequest>(json!({
            "external_user_id": "user_1", "n": limit
        }));
        assert_eq!(decoded.is_ok(), (1..=100).contains(&limit));
    }
}

#[test]
fn workflow_request_debug_reports_body_size_without_body_bytes() {
    let request = WorkflowRequest {
        body: Some(vec![0, 1, 2, 255]),
        content_type: Some("application/octet-stream".into()),
    };
    let debug = format!("{request:?}");
    assert!(debug.contains("body_bytes: Some(4)"), "{debug}");
    assert!(!debug.contains("0, 1, 2, 255"), "{debug}");
}
