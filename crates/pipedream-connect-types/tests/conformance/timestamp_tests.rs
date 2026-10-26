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

#[test]
fn integer_date_fields_follow_the_openapi_epoch_unit() {
    let document: Value = serde_json::from_str(&fs::read_to_string(FIXTURE).unwrap()).unwrap();
    let seconds = 1_700_000_000;
    assert_epoch_field::<UsageRecord>(
        &document,
        "ConnectUsage",
        "usage_start_ts",
        json!({"credits_used":1,"usage_start_ts":seconds,"usage_end_ts":seconds}),
        |value| &value.usage_start_ts,
        seconds,
    );
    assert_epoch_field::<UsageRecord>(
        &document,
        "ConnectUsage",
        "usage_end_ts",
        json!({"credits_used":1,"usage_start_ts":seconds,"usage_end_ts":seconds}),
        |value| &value.usage_end_ts,
        seconds,
    );
    for schema in [
        "Webhook",
        "WebhookWithOptionalSigningKey",
        "WebhookWithSigningKey",
    ] {
        for field in ["created_at", "updated_at"] {
            match schema {
                "Webhook" => assert_epoch_field(
                    &document,
                    schema,
                    field,
                    json!({
                        "id":"wh_sample","url":"https://example.com","signing_key_set":false,
                        "created_at":seconds,"updated_at":seconds
                    }),
                    if field == "created_at" {
                        (|value: &Webhook| &value.created_at) as fn(&Webhook) -> &Timestamp
                    } else {
                        (|value: &Webhook| &value.updated_at) as fn(&Webhook) -> &Timestamp
                    },
                    seconds,
                ),
                "WebhookWithOptionalSigningKey" => assert_epoch_field(
                    &document,
                    schema,
                    field,
                    json!({
                        "id":"wh_sample","url":"https://example.com","signing_key_set":false,
                        "created_at":seconds,"updated_at":seconds
                    }),
                    if field == "created_at" {
                        (|value: &WebhookWithOptionalSigningKey| &value.created_at)
                            as fn(&WebhookWithOptionalSigningKey) -> &Timestamp
                    } else {
                        (|value: &WebhookWithOptionalSigningKey| &value.updated_at)
                            as fn(&WebhookWithOptionalSigningKey) -> &Timestamp
                    },
                    seconds,
                ),
                "WebhookWithSigningKey" => assert_epoch_field(
                    &document,
                    schema,
                    field,
                    json!({
                        "id":"wh_sample","url":"https://example.com","signing_key":"secret",
                        "signing_key_set":true,"created_at":seconds,"updated_at":seconds
                    }),
                    if field == "created_at" {
                        (|value: &WebhookWithSigningKey| &value.created_at)
                            as fn(&WebhookWithSigningKey) -> &Timestamp
                    } else {
                        (|value: &WebhookWithSigningKey| &value.updated_at)
                            as fn(&WebhookWithSigningKey) -> &Timestamp
                    },
                    seconds,
                ),
                _ => unreachable!(),
            }
        }
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
