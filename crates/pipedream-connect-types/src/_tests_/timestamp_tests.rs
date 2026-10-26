//! Date fields accept both wire timestamp forms and preserve that form.

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::Timestamp;
use crate::{
    Account, AppOverride, ConnectTokenResponse, EmittedEvent, Emitter, ExternalUser, HttpInterface,
    OauthApp, Observation, TimerInterface, UsageRecord, Webhook, WebhookWithOptionalSigningKey,
    WebhookWithSigningKey,
};

const RFC3339: &str = "2026-01-01T00:00:00Z";
const MILLIS: i64 = 1_767_225_600_000;

fn assert_forms<T>(mut sample: Value, fields: &[&str])
where
    T: DeserializeOwned + Serialize,
{
    for field in fields {
        for wire in [json!(RFC3339), json!(MILLIS)] {
            sample[field] = wire.clone();
            let decoded: T = serde_json::from_value(sample.clone()).unwrap();
            assert_eq!(
                serde_json::to_value(decoded).unwrap()[field],
                wire,
                "{field}"
            );
        }
    }
}

#[test]
fn account_and_administration_dates_preserve_both_forms() {
    assert_forms::<Account>(
        json!({"id":"apn_sample"}),
        &[
            "created_at",
            "updated_at",
            "expires_at",
            "last_refreshed_at",
            "next_refresh_at",
        ],
    );
    assert_forms::<ExternalUser>(
        json!({"id":"user_1","external_id":"customer_1"}),
        &["created_at", "updated_at"],
    );
    assert_forms::<ConnectTokenResponse>(
        json!({"connect_link_url":"https://example.com","token":"ctok_00000000000000000000000000000000"}),
        &["expires_at"],
    );
    assert_forms::<AppOverride>(
        json!({"id":"override_1","name":"Override","app":{"name_slug":"github","name":"GitHub","img_src":"https://example.com"},"owner_id":"owner"}),
        &["created_at", "updated_at"],
    );
    assert_forms::<OauthApp>(
        json!({"id":"oa_sample","client_id":"client","owner_id":"owner"}),
        &["created_at", "updated_at"],
    );
}

#[test]
fn integer_schema_dates_preserve_integer_output() {
    assert_forms::<Webhook>(
        json!({"id":"wh_sample","url":"https://example.com","signing_key_set":false,"created_at":0,"updated_at":0}),
        &["created_at", "updated_at"],
    );
    assert_forms::<UsageRecord>(
        json!({"credits_used":1,"usage_start_ts":0,"usage_end_ts":0}),
        &["usage_start_ts", "usage_end_ts"],
    );
    assert_forms::<EmittedEvent>(json!({"e":{},"k":"emit","ts":0,"id":"event_1"}), &["ts"]);
}

#[test]
fn epoch_seconds_schema_dates_decode_as_seconds_and_preserve_input() {
    let seconds = 1_700_000_000;
    assert_seconds(
        json!({"credits_used":1,"usage_start_ts":seconds,"usage_end_ts":seconds}),
        "usage_start_ts",
        |value: &UsageRecord| &value.usage_start_ts,
        seconds,
    );
    assert_seconds(
        json!({"credits_used":1,"usage_start_ts":seconds,"usage_end_ts":seconds}),
        "usage_end_ts",
        |value: &UsageRecord| &value.usage_end_ts,
        seconds,
    );
    for field in ["created_at", "updated_at"] {
        assert_seconds(
            json!({
                "id":"wh_sample","url":"https://example.com","signing_key_set":false,
                "created_at":seconds,"updated_at":seconds
            }),
            field,
            if field == "created_at" {
                (|value: &Webhook| &value.created_at) as fn(&Webhook) -> &Timestamp
            } else {
                (|value: &Webhook| &value.updated_at) as fn(&Webhook) -> &Timestamp
            },
            seconds,
        );
        assert_seconds(
            json!({
                "id":"wh_sample","url":"https://example.com","signing_key_set":false,
                "created_at":seconds,"updated_at":seconds
            }),
            field,
            if field == "created_at" {
                (|value: &WebhookWithOptionalSigningKey| &value.created_at)
                    as fn(&WebhookWithOptionalSigningKey) -> &Timestamp
            } else {
                (|value: &WebhookWithOptionalSigningKey| &value.updated_at)
                    as fn(&WebhookWithOptionalSigningKey) -> &Timestamp
            },
            seconds,
        );
        assert_seconds(
            json!({
                "id":"wh_sample","url":"https://example.com","signing_key":"secret",
                "signing_key_set":true,"created_at":seconds,"updated_at":seconds
            }),
            field,
            if field == "created_at" {
                (|value: &WebhookWithSigningKey| &value.created_at)
                    as fn(&WebhookWithSigningKey) -> &Timestamp
            } else {
                (|value: &WebhookWithSigningKey| &value.updated_at)
                    as fn(&WebhookWithSigningKey) -> &Timestamp
            },
            seconds,
        );
    }
}

fn assert_seconds<T>(
    sample: Value,
    field: &str,
    timestamp: fn(&T) -> &Timestamp,
    expected_seconds: i64,
) where
    T: DeserializeOwned + Serialize,
{
    let decoded: T = serde_json::from_value(sample).unwrap();
    assert_eq!(
        timestamp(&decoded).as_datetime().timestamp(),
        expected_seconds
    );
    assert_eq!(
        serde_json::to_value(decoded).unwrap()[field],
        json!(expected_seconds)
    );
}

#[test]
fn trigger_and_execution_dates_preserve_both_forms() {
    assert_forms::<Emitter>(
        json!({"type":"DeployedComponent","id":"dc_sample","owner_id":"owner","component_id":"component","configurable_props":[],"configured_props":{},"active":true,"created_at":0,"updated_at":0,"name":"Sample","name_slug":"sample"}),
        &["created_at", "updated_at"],
    );
    assert_forms::<HttpInterface>(
        json!({"id":"hi_sample","key":"http","endpoint_url":"https://example.com","custom_response":false,"created_at":0,"updated_at":0,"type":"HttpInterface"}),
        &["created_at", "updated_at"],
    );
    assert_forms::<TimerInterface>(
        json!({"id":"ti_sample","timezone":"UTC","schedule_changed_at":0,"created_at":0,"updated_at":0,"type":"TimerInterface"}),
        &["schedule_changed_at", "created_at", "updated_at"],
    );
    assert_forms::<Observation>(json!({"k":"console.log","ts":0}), &["ts"]);
}

#[test]
fn invalid_model_dates_are_rejected() {
    assert!(
        serde_json::from_value::<Account>(json!({
            "id": "apn_sample",
            "created_at": "bad"
        }))
        .is_err()
    );
    assert!(
        serde_json::from_value::<EmittedEvent>(json!({
            "e": {},
            "k": "emit",
            "ts": true,
            "id": "event_1"
        }))
        .is_err()
    );
    assert!(
        serde_json::from_value::<Observation>(json!({
            "k": "console.log",
            "ts": "bad"
        }))
        .is_err()
    );
}
