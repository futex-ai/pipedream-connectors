//! OpenAPI date-unit conformance checks.

use std::fs;

use chrono::{TimeZone, Utc};
use pipedream_connect_types::*;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/pipedream_openapi_swagger.json"
);

type TimestampField<T> = (&'static str, fn(&T) -> &Timestamp);

#[test]
fn integer_date_fields_follow_the_openapi_epoch_unit() {
    let document: Value = serde_json::from_str(&fs::read_to_string(FIXTURE).unwrap()).unwrap();
    let seconds = 1_700_000_000;
    assert_epoch_schema::<UsageRecord>(
        &document,
        "ConnectUsage",
        json!({"credits_used":1,"usage_start_ts":seconds,"usage_end_ts":seconds}),
        &[
            ("usage_start_ts", |value: &UsageRecord| {
                &value.usage_start_ts
            }),
            ("usage_end_ts", |value: &UsageRecord| &value.usage_end_ts),
        ],
        seconds,
    );

    let webhook_sample = json!({
        "id":"wh_sample","url":"https://example.com","signing_key_set":false,
        "created_at":seconds,"updated_at":seconds
    });
    assert_epoch_schema::<Webhook>(
        &document,
        "Webhook",
        webhook_sample.clone(),
        &[
            ("created_at", |value: &Webhook| &value.created_at),
            ("updated_at", |value: &Webhook| &value.updated_at),
        ],
        seconds,
    );
    assert_epoch_schema::<WebhookWithOptionalSigningKey>(
        &document,
        "WebhookWithOptionalSigningKey",
        webhook_sample.clone(),
        &[
            ("created_at", |value: &WebhookWithOptionalSigningKey| {
                &value.created_at
            }),
            ("updated_at", |value: &WebhookWithOptionalSigningKey| {
                &value.updated_at
            }),
        ],
        seconds,
    );
    let mut signed_webhook_sample = webhook_sample;
    signed_webhook_sample["signing_key"] = json!("secret");
    signed_webhook_sample["signing_key_set"] = json!(true);
    assert_epoch_schema::<WebhookWithSigningKey>(
        &document,
        "WebhookWithSigningKey",
        signed_webhook_sample,
        &[
            ("created_at", |value: &WebhookWithSigningKey| {
                &value.created_at
            }),
            ("updated_at", |value: &WebhookWithSigningKey| {
                &value.updated_at
            }),
        ],
        seconds,
    );

    let milliseconds = 1_700_000_000_000;
    assert_epoch_schema::<DeployedComponent>(
        &document,
        "DeployedComponent",
        json!({
            "id":"dc_sample","owner_id":"owner","component_id":"component",
            "configurable_props":[],"configured_props":{},"active":true,
            "created_at":0,"updated_at":0,"name":"Sample","name_slug":"sample",
            "type":"DeployedComponent"
        }),
        &[
            ("created_at", |value: &DeployedComponent| &value.created_at),
            ("updated_at", |value: &DeployedComponent| &value.updated_at),
        ],
        milliseconds,
    );
    assert_epoch_schema::<HttpInterface>(
        &document,
        "HttpInterface",
        json!({
            "id":"hi_sample","key":"http","endpoint_url":"https://example.com",
            "custom_response":false,"created_at":0,"updated_at":0,"type":"HttpInterface"
        }),
        &[
            ("created_at", |value: &HttpInterface| &value.created_at),
            ("updated_at", |value: &HttpInterface| &value.updated_at),
        ],
        milliseconds,
    );
    assert_epoch_schema::<TimerInterface>(
        &document,
        "TimerInterface",
        json!({
            "id":"ti_sample","timezone":"UTC","schedule_changed_at":0,
            "created_at":0,"updated_at":0,"type":"TimerInterface"
        }),
        &[
            ("created_at", |value: &TimerInterface| &value.created_at),
            ("updated_at", |value: &TimerInterface| &value.updated_at),
        ],
        milliseconds,
    );
    assert_epoch_schema::<EmittedEvent>(
        &document,
        "EmittedEvent",
        json!({"e":{},"k":"emit","ts":0,"id":"event_1"}),
        &[("ts", |value: &EmittedEvent| &value.ts)],
        milliseconds,
    );
}

fn assert_epoch_schema<T>(
    document: &Value,
    schema: &str,
    sample: Value,
    fields: &[TimestampField<T>],
    raw: i64,
) where
    T: DeserializeOwned + Serialize,
{
    for (field, timestamp) in fields {
        assert_epoch_field(document, schema, field, sample.clone(), *timestamp, raw);
    }
}

fn assert_epoch_field<T>(
    document: &Value,
    schema: &str,
    field: &str,
    mut sample: Value,
    timestamp: fn(&T) -> &Timestamp,
    raw: i64,
) where
    T: DeserializeOwned + Serialize,
{
    let property = &document["components"]["schemas"][schema]["properties"][field];
    assert_eq!(property["type"], "integer", "{schema}.{field}");
    let description = property["description"]
        .as_str()
        .unwrap_or_else(|| panic!("missing description for {schema}.{field}"));
    let divisor =
        if description.contains("epoch seconds") || description.contains("timestamp (seconds)") {
            1
        } else if description.contains("epoch milliseconds") {
            1_000
        } else {
            panic!("missing epoch unit for {schema}.{field}: {description}");
        };
    sample[field] = json!(raw);
    let decoded: T = serde_json::from_value(sample).unwrap();
    let expected = Utc
        .timestamp_opt(raw / divisor, 0)
        .single()
        .unwrap_or_else(|| panic!("invalid expected instant for {schema}.{field}"));
    assert_eq!(
        timestamp(&decoded).as_datetime(),
        expected,
        "{schema}.{field}"
    );
    assert_eq!(serde_json::to_value(decoded).unwrap()[field], json!(raw));
}
