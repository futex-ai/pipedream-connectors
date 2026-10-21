//! Response shapes, optional response values, and unknown fields.

use serde_json::json;

use crate::{
    Account, App, AppOverride, CreateAccountResponse, ErrorMessage, ErrorResponse,
    GetAccountResponse, GetAccountsResponse, Project, ProjectInfo, ValidateTokenResponse,
};

#[test]
fn account_operations_use_the_documented_bare_or_paged_shapes() {
    let account = json!({"id":"apn_sample"});
    let get: GetAccountResponse = serde_json::from_value(account.clone()).unwrap();
    let created: CreateAccountResponse = serde_json::from_value(account.clone()).unwrap();
    assert_eq!(get, created);
    let list: GetAccountsResponse =
        serde_json::from_value(json!({"data":[account],"page_info":{}})).unwrap();
    assert_eq!(list.data[0].id.as_str(), "apn_sample");
}

#[test]
fn response_absence_does_not_fabricate_false_or_empty_values() {
    let account: Account = serde_json::from_value(json!({"id":"apn_sample"})).unwrap();
    assert_eq!(account.healthy, None);
    assert_eq!(
        serde_json::to_value(account).unwrap()["healthy"],
        json!(null)
    );
    let project: Project =
        serde_json::from_value(json!({"id":"proj_sample","name":"Example"})).unwrap();
    assert_eq!(project.connect_require_key_auth_test, None);
    let validation: ValidateTokenResponse =
        serde_json::from_value(json!({"error":null,"success":true})).unwrap();
    assert_eq!(validation.oauth_app_workday_official, None);
}

#[test]
fn project_info_ids_and_proxy_domains_accept_provider_shapes() {
    let info: ProjectInfo =
        serde_json::from_value(json!({"apps":[{"name_slug":"github","id":"app_OkrhR1"}]})).unwrap();
    assert_eq!(info.apps[0].id.as_deref(), Some("app_OkrhR1"));
    let sample = json!({"name_slug":"github","name":"GitHub","img_src":"https://example.com/github.png","id":null,"auth_type":null,"description":null,"custom_fields_json":null,"categories":[],"featured_weight":1.0,"scope_profiles":[],"connect":{"proxy_enabled":true,"allowed_domains":null,"base_proxy_target_url":null,"future":true},"future_field":{"kept":true}});
    let app: App = serde_json::from_value(sample.clone()).unwrap();
    assert_eq!(app.extra.get("future_field"), sample.get("future_field"));
    assert_eq!(app.connect.as_ref().unwrap().allowed_domains, None);
    assert_eq!(serde_json::to_value(app).unwrap(), sample);
}

#[test]
fn error_envelopes_preserve_text_and_structured_errors() {
    for error in [
        json!("denied"),
        json!({"domain":["not allowed for this app"]}),
    ] {
        let envelope: ErrorResponse = serde_json::from_value(json!({"error":error})).unwrap();
        assert_eq!(
            matches!(&envelope.error, ErrorMessage::Text(_)),
            error.is_string()
        );
        assert_eq!(serde_json::to_value(envelope).unwrap()["error"], error);
    }
}

#[test]
fn omitted_catalog_defaults_remain_unknown_instead_of_fabricated() {
    let app: App = serde_json::from_value(
        json!({"name_slug":"github","name":"GitHub","img_src":"https://example.com/logo.png"}),
    )
    .unwrap();
    let encoded = serde_json::to_value(app).unwrap();
    for field in ["categories", "featured_weight", "scope_profiles"] {
        assert!(
            encoded[field].is_null(),
            "fabricated {field}: {}",
            encoded[field]
        );
    }
    let model:AppOverride=serde_json::from_value(json!({"id":"ao_sample","name":"Example","app":{"name_slug":"github","name":"GitHub","img_src":"https://example.com/logo.png"},"owner_id":"sample"})).unwrap();
    assert!(serde_json::to_value(model).unwrap()["cfmap"].is_null());
}
