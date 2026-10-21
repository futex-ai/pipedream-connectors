//! Tests that every credential-class field of a printed model is redacted.

use pipedream_connect_types::{
    Account, App, Data, Emitter, Page, ProjectWebhook, TriggerWebhooks,
    WebhookWithOptionalSigningKey, WebhookWithSigningKey,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::redact::{REDACTED, Redact};

fn redacted<T: DeserializeOwned + Serialize + Redact>(raw: Value) -> Value {
    let mut model: T = serde_json::from_value(raw).unwrap();
    model.redact();
    serde_json::to_value(model).unwrap()
}

fn account(credentials: Value) -> Value {
    json!({
        "id": "apn_Account1",
        "name": "Work calendar",
        "external_id": "user_1",
        "healthy": true,
        "dead": false,
        "credentials": credentials,
        "error": null,
        "last_refreshed_at": null,
        "next_refresh_at": null,
        "custom_field": "kept",
    })
}

fn webhook(signing_key: Option<&str>) -> Value {
    let mut webhook =
        json!({"id": "hook_1", "url": "https://example.com/hook", "signing_key_set": true});
    if let Some(key) = signing_key {
        webhook["signing_key"] = json!(key);
    }
    webhook
}

fn deployed_trigger() -> Value {
    json!({
        "type": "DeployedComponent",
        "id": "dc_Trigger1",
        "owner_id": "exu_1",
        "component_id": "sc_1",
        "configurable_props": [],
        "configured_props": {"calendarId": "primary"},
        "active": true,
        "created_at": 1_700_000_000_000_i64,
        "updated_at": 1_700_000_000_000_i64,
        "name": "New event",
        "name_slug": "new-event",
        "callback_observations": null,
        "webhook_signing_key": "trigger-key",
        "webhooks": [webhook(Some("hook-key"))],
    })
}

#[test]
fn account_credentials_are_redacted_and_other_fields_are_kept() {
    let output = redacted::<Account>(account(json!({"oauth_access_token": "token-value"})));
    assert_eq!(output, account(json!(REDACTED)));
}

#[test]
fn account_without_credentials_is_unchanged() {
    assert_eq!(
        redacted::<Account>(account(Value::Null)),
        account(Value::Null)
    );
}

#[test]
fn every_account_in_a_page_is_redacted() {
    let page = json!({
        "data": [account(json!({"key": "one"})), account(json!({"key": "two"}))],
        "page_info": {"count": 2, "start_cursor": null, "end_cursor": "c2"},
    });
    let output = redacted::<Page<Account>>(page);
    for account in output["data"].as_array().unwrap() {
        assert_eq!(account["credentials"], json!(REDACTED));
    }
    assert_eq!(output["page_info"]["end_cursor"], json!("c2"));
}

#[test]
fn deployed_trigger_signing_keys_are_redacted() {
    let output = redacted::<Data<Emitter>>(json!({"data": deployed_trigger()}));
    assert_eq!(output["data"]["webhook_signing_key"], json!(REDACTED));
    assert_eq!(
        output["data"]["webhooks"][0]["signing_key"],
        json!(REDACTED)
    );
    assert_eq!(
        output["data"]["webhooks"][0]["url"],
        json!("https://example.com/hook")
    );
    assert_eq!(
        output["data"]["configured_props"],
        json!({"calendarId": "primary"})
    );
}

#[test]
fn other_emitters_keep_their_fields() {
    let http = json!({
        "type": "HttpInterface",
        "id": "hi_1",
        "key": "endpoint",
        "endpoint_url": "https://example.m.pipedream.net",
        "custom_response": false,
        "created_at": 1_700_000_000_000_i64,
        "updated_at": 1_700_000_000_000_i64,
    });
    assert_eq!(redacted::<Emitter>(http.clone()), http);
    let future = json!({"type": "FutureInterface", "id": "fi_1"});
    assert_eq!(redacted::<Emitter>(future.clone()), future);
}

#[test]
fn trigger_webhook_signing_keys_are_redacted() {
    let raw = json!({
        "webhook_urls": ["https://example.com/hook"],
        "webhooks": [webhook(Some("hook-key")), webhook(None)],
    });
    let output = redacted::<TriggerWebhooks>(raw);
    assert_eq!(output["webhooks"][0]["signing_key"], json!(REDACTED));
    assert!(output["webhooks"][1].get("signing_key").is_none());
}

#[test]
fn project_webhook_signing_keys_are_redacted() {
    let mut raw = webhook(Some("project-key"));
    raw["created_at"] = json!(1_700_000_000_i64);
    raw["updated_at"] = json!(1_700_000_000_i64);
    let output = redacted::<ProjectWebhook>(raw.clone());
    assert_eq!(output["signing_key"], json!(REDACTED));
    let output = redacted::<Data<WebhookWithSigningKey>>(json!({"data": raw.clone()}));
    assert_eq!(output["data"]["signing_key"], json!(REDACTED));
    let output = redacted::<WebhookWithOptionalSigningKey>(raw);
    assert_eq!(output["signing_key"], json!(REDACTED));
}

#[test]
fn models_without_credentials_are_unchanged() {
    let app = json!({
        "id": "app_1",
        "name_slug": "google_calendar",
        "name": "Google Calendar",
        "auth_type": "oauth",
        "description": null,
        "img_src": "https://example.com/logo.png",
        "custom_fields_json": null,
    });
    assert_eq!(redacted::<App>(app.clone()), app);
}
