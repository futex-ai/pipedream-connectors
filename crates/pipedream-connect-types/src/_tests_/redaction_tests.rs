//! Table tests for credential-class diagnostics at every nesting level.

use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::{
    Account, ConnectToken, ConnectTokenResponse, CreateAccountRequest, CreateOauthAppRequest,
    CreateOauthTokenRequest, Data, DeployedComponent, OAuthTokenResponse, Page, RateLimitToken,
    SecretJson, SecretString, TriggerWebhook, TriggerWebhooks, UpdateOauthAppRequest,
    WebhookWithOptionalSigningKey, WebhookWithSigningKey,
};

const MARKER: &str = "0123456789abcdef0123456789abcdef";
type Check = fn(Value);

fn redacted<T: DeserializeOwned + std::fmt::Debug>(sample: Value) {
    let value: T = serde_json::from_value(sample).unwrap();
    assert!(!format!("{value:?}").contains(MARKER));
}

#[test]
fn every_credential_field_and_nested_model_redacts_debug() {
    let webhook = json!({"id":"wh_sample","url":"https://example.com","signing_key_set":true,"signing_key":MARKER,"created_at":0,"updated_at":0});
    let account =
        json!({"id":"apn_sample","credentials":{"api_key":MARKER,"nested":{"password":MARKER}}});
    let deployment = json!({"id":"dc_sample","owner_id":"sample","component_id":"sample","configurable_props":[],"configured_props":{},"active":true,"created_at":0,"updated_at":0,"name":"Sample","name_slug":"sample","type":"DeployedComponent","webhook_signing_key":MARKER,"webhooks":[webhook],"webhook_urls":["https://example.com"]});
    let cases: Vec<(&str, Check, Value)> = vec![
        ("SecretString", redacted::<SecretString>, json!(MARKER)),
        (
            "SecretJson",
            redacted::<SecretJson>,
            json!({"api_key":MARKER}),
        ),
        (
            "ConnectToken",
            redacted::<ConnectToken>,
            json!(format!("ctok_{MARKER}")),
        ),
        ("Account.credentials", redacted::<Account>, account.clone()),
        (
            "CreateAccountRequest.cfmap_json and token",
            redacted::<CreateAccountRequest>,
            json!({"app_slug":"demo","cfmap_json":MARKER,"connect_token":format!("ctok_{MARKER}")}),
        ),
        (
            "CreateOauthAppRequest.client_secret",
            redacted::<CreateOauthAppRequest>,
            json!({"app":"demo","client_id":"id","client_secret":MARKER}),
        ),
        (
            "UpdateOauthAppRequest.client_secret",
            redacted::<UpdateOauthAppRequest>,
            json!({"client_secret":MARKER}),
        ),
        (
            "CreateOauthTokenRequest.client_secret",
            redacted::<CreateOauthTokenRequest>,
            json!({"grant_type":"client_credentials","client_id":"id","client_secret":MARKER}),
        ),
        (
            "OAuthTokenResponse.access_token",
            redacted::<OAuthTokenResponse>,
            json!({"access_token":MARKER,"token_type":"Bearer","expires_in":3600}),
        ),
        (
            "RateLimitToken.token",
            redacted::<RateLimitToken>,
            json!({"token":MARKER}),
        ),
        (
            "ConnectTokenResponse token and link",
            redacted::<ConnectTokenResponse>,
            json!({"token":format!("ctok_{MARKER}"),"connect_link_url":format!("https://example.com/{MARKER}"),"expires_at":"2026-01-01T00:00:00Z"}),
        ),
        (
            "TriggerWebhook.signing_key",
            redacted::<TriggerWebhook>,
            webhook.clone(),
        ),
        (
            "WebhookWithOptionalSigningKey.signing_key",
            redacted::<WebhookWithOptionalSigningKey>,
            webhook.clone(),
        ),
        (
            "WebhookWithSigningKey.signing_key",
            redacted::<WebhookWithSigningKey>,
            webhook.clone(),
        ),
        (
            "TriggerWebhooks nested signing_key",
            redacted::<TriggerWebhooks>,
            json!({"webhook_urls":["https://example.com"],"webhooks":[webhook]}),
        ),
        (
            "DeployedComponent nested signing keys",
            redacted::<DeployedComponent>,
            deployment,
        ),
        (
            "Data nested credentials",
            redacted::<Data<Account>>,
            json!({"data":account}),
        ),
        (
            "Page nested credentials",
            redacted::<Page<Account>>,
            json!({"data":[account],"page_info":{}}),
        ),
    ];
    for (name, check, sample) in cases {
        assert!(std::panic::catch_unwind(|| check(sample)).is_ok(), "{name}");
    }
}

#[test]
fn secrets_keep_explicit_access_and_wire_values() {
    let secret = SecretString::new("client-secret");
    assert_eq!(format!("{secret:?}"), "[REDACTED]");
    assert_eq!(secret.expose_secret(), "client-secret");
    assert_eq!(
        serde_json::to_value(secret).unwrap(),
        json!("client-secret")
    );
    let value = SecretJson::new(json!({"token":"value"}));
    assert_eq!(format!("{value:?}"), "[REDACTED]");
    assert_eq!(value.expose_secret(), &json!({"token":"value"}));
}
