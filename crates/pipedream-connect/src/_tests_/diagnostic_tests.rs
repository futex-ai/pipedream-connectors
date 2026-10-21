//! Safe error details and decode caller provenance remain observable.

use std::error::Error;
use std::time::Duration;

use pipedream_connect_types::ValidationField;
use serde_json::json;

use crate::error::{self, InvalidRequest};

use super::support::response;

#[test]
fn invalid_requests_show_the_safe_reason_and_preserve_the_source() {
    let error = error::Error::InvalidRequest(InvalidRequest::Origin);
    assert!(error.to_string().contains("bare HTTPS origin"));
    assert!(error.source().is_some());
}

#[test]
fn model_validation_messages_include_the_safe_field_and_bounds() {
    let error = error::Error::InvalidRequest(InvalidRequest::Model(
        pipedream_connect_types::Error::OutOfRange {
            field: ValidationField::ExpiresIn,
            minimum: 1,
            maximum: 14_400,
        },
    ));
    let message = error.to_string();
    assert!(message.contains("expires_in must be between 1 and 14400"));
}

#[test]
fn typed_debug_exposes_safe_fields_and_hides_provider_text() {
    let error = error::Error::RateLimited {
        retry_after: Some(Duration::from_secs(12)),
        limit: Some(100),
        remaining: Some(0),
        reset: Some(1200),
    };
    let debug = format!("{error:?}");
    for field in [
        "retry_after: Some(12s)",
        "limit: Some(100)",
        "remaining: Some(0)",
        "reset: Some(1200)",
    ] {
        assert!(debug.contains(field), "missing {field}: {debug}");
    }
    let execution = serde_json::from_value(json!({
        "name":"secret-name", "message":"secret-message",
        "attribution":{"origin":"upstream_api", "network_calls_count":2,
            "last_call":{"status":401,"duration_ms":12.5,"host":"secret-host","extra":"secret"},
            "extra":"secret"},"extra":"secret"
    }))
    .unwrap();
    let debug = format!("{:?}", error::Error::ExecutionFailed(Box::new(execution)));
    for safe in [
        "UpstreamApi",
        "network_calls_count: Some(2)",
        "status: Some(401)",
        "duration_ms: Some(12.5)",
    ] {
        assert!(debug.contains(safe), "missing {safe}: {debug}");
    }
    assert!(!debug.contains("secret"));
}

#[test]
fn response_decode_records_its_actual_caller() {
    let reply = response(200, json!({"wrong":"secret"}));
    let call_line = line!() + 1;
    let error = reply.json::<u32>().unwrap_err();
    let error::Error::Internal(source) = error else {
        panic!("expected internal failure")
    };
    assert_eq!(source.caller_at().file(), file!());
    assert_eq!(source.caller_at().line(), call_line);
}

#[test]
fn unknown_execution_origin_and_provider_codes_stay_out_of_debug() {
    let execution = serde_json::from_value(json!({"attribution":{"origin":"secret-origin"},"name":"secret-name","message":"secret-message"})).unwrap();
    let rejected = error::Error::Rejected {
        status: 422,
        code: Some("secret-code".into()),
        message: Some("secret-message".into()),
    };
    let execution = error::Error::ExecutionFailed(Box::new(execution));
    for error in [rejected, execution] {
        assert!(!format!("{error:?} {error}").contains("secret"));
    }
}
