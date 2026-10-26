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

struct EpochField<T> {
    schema: &'static str,
    field: &'static str,
    sample: Value,
    timestamp: fn(&T) -> &Timestamp,
    raw: i64,
}

#[test]
fn integer_date_fields_follow_the_openapi_epoch_unit() {
    let document: Value = serde_json::from_str(&fs::read_to_string(FIXTURE).unwrap()).unwrap();
    let seconds = 1_700_000_000;
    assert_epoch_fields(
        &document,
        [
            EpochField {
                schema: "ConnectUsage",
                field: "usage_start_ts",
                sample: json!({"credits_used":1,"usage_start_ts":seconds,"usage_end_ts":seconds}),
                timestamp: |value: &UsageRecord| &value.usage_start_ts,
                raw: seconds,
            },
            EpochField {
                schema: "ConnectUsage",
                field: "usage_end_ts",
                sample: json!({"credits_used":1,"usage_start_ts":seconds,"usage_end_ts":seconds}),
                timestamp: |value: &UsageRecord| &value.usage_end_ts,
                raw: seconds,
            },
        ],
    );
    assert_epoch_fields(
        &document,
        [
            EpochField {
                schema: "Webhook",
                field: "created_at",
                sample: json!({
                    "id":"wh_sample","url":"https://example.com","signing_key_set":false,
                    "created_at":seconds,"updated_at":seconds
                }),
                timestamp: |value: &Webhook| &value.created_at,
                raw: seconds,
            },
            EpochField {
                schema: "Webhook",
                field: "updated_at",
                sample: json!({
                    "id":"wh_sample","url":"https://example.com","signing_key_set":false,
                    "created_at":seconds,"updated_at":seconds
                }),
                timestamp: |value: &Webhook| &value.updated_at,
                raw: seconds,
            },
        ],
    );
    assert_epoch_fields(
        &document,
        [
            EpochField {
                schema: "WebhookWithOptionalSigningKey",
                field: "created_at",
                sample: json!({
                    "id":"wh_sample","url":"https://example.com","signing_key_set":false,
                    "created_at":seconds,"updated_at":seconds
                }),
                timestamp: |value: &WebhookWithOptionalSigningKey| &value.created_at,
                raw: seconds,
            },
            EpochField {
                schema: "WebhookWithOptionalSigningKey",
                field: "updated_at",
                sample: json!({
                    "id":"wh_sample","url":"https://example.com","signing_key_set":false,
                    "created_at":seconds,"updated_at":seconds
                }),
                timestamp: |value: &WebhookWithOptionalSigningKey| &value.updated_at,
                raw: seconds,
            },
        ],
    );
    assert_epoch_fields(
        &document,
        [
            EpochField {
                schema: "WebhookWithSigningKey",
                field: "created_at",
                sample: json!({
                    "id":"wh_sample","url":"https://example.com","signing_key":"secret",
                    "signing_key_set":true,"created_at":seconds,"updated_at":seconds
                }),
                timestamp: |value: &WebhookWithSigningKey| &value.created_at,
                raw: seconds,
            },
            EpochField {
                schema: "WebhookWithSigningKey",
                field: "updated_at",
                sample: json!({
                    "id":"wh_sample","url":"https://example.com","signing_key":"secret",
                    "signing_key_set":true,"created_at":seconds,"updated_at":seconds
                }),
                timestamp: |value: &WebhookWithSigningKey| &value.updated_at,
                raw: seconds,
            },
        ],
    );

    let milliseconds = 1_700_000_000_000;
    assert_epoch_fields(
        &document,
        [
            EpochField {
                schema: "DeployedComponent",
                field: "created_at",
                sample: json!({
                    "id":"dc_sample","owner_id":"owner","component_id":"component",
                    "configurable_props":[],"configured_props":{},"active":true,
                    "created_at":0,"updated_at":0,"name":"Sample","name_slug":"sample",
                    "type":"DeployedComponent"
                }),
                timestamp: |value: &DeployedComponent| &value.created_at,
                raw: milliseconds,
            },
            EpochField {
                schema: "DeployedComponent",
                field: "updated_at",
                sample: json!({
                    "id":"dc_sample","owner_id":"owner","component_id":"component",
                    "configurable_props":[],"configured_props":{},"active":true,
                    "created_at":0,"updated_at":0,"name":"Sample","name_slug":"sample",
                    "type":"DeployedComponent"
                }),
                timestamp: |value: &DeployedComponent| &value.updated_at,
                raw: milliseconds,
            },
        ],
    );
    assert_epoch_fields(
        &document,
        [
            EpochField {
                schema: "HttpInterface",
                field: "created_at",
                sample: json!({
                    "id":"hi_sample","key":"http","endpoint_url":"https://example.com",
                    "custom_response":false,"created_at":0,"updated_at":0,"type":"HttpInterface"
                }),
                timestamp: |value: &HttpInterface| &value.created_at,
                raw: milliseconds,
            },
            EpochField {
                schema: "HttpInterface",
                field: "updated_at",
                sample: json!({
                    "id":"hi_sample","key":"http","endpoint_url":"https://example.com",
                    "custom_response":false,"created_at":0,"updated_at":0,"type":"HttpInterface"
                }),
                timestamp: |value: &HttpInterface| &value.updated_at,
                raw: milliseconds,
            },
        ],
    );
    assert_epoch_fields(
        &document,
        [
            EpochField {
                schema: "TimerInterface",
                field: "created_at",
                sample: json!({
                    "id":"ti_sample","timezone":"UTC","schedule_changed_at":0,
                    "created_at":0,"updated_at":0,"type":"TimerInterface"
                }),
                timestamp: |value: &TimerInterface| &value.created_at,
                raw: milliseconds,
            },
            EpochField {
                schema: "TimerInterface",
                field: "updated_at",
                sample: json!({
                    "id":"ti_sample","timezone":"UTC","schedule_changed_at":0,
                    "created_at":0,"updated_at":0,"type":"TimerInterface"
                }),
                timestamp: |value: &TimerInterface| &value.updated_at,
                raw: milliseconds,
            },
        ],
    );
    assert_epoch_fields(
        &document,
        [EpochField {
            schema: "EmittedEvent",
            field: "ts",
            sample: json!({"e":{},"k":"emit","ts":0,"id":"event_1"}),
            timestamp: |value: &EmittedEvent| &value.ts,
            raw: milliseconds,
        }],
    );
}

fn assert_epoch_fields<T>(document: &Value, cases: impl IntoIterator<Item = EpochField<T>>)
where
    T: DeserializeOwned + Serialize,
{
    for case in cases {
        assert_epoch_field(
            document,
            case.schema,
            case.field,
            case.sample,
            case.timestamp,
            case.raw,
        );
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
