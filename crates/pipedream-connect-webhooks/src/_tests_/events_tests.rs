//! Fixed payload tests for connection event decoding.

use serde_json::json;

use crate::{ConnectionEvent, ConnectionEventKind, PayloadError};

const TOKEN: &str = "ctok_0123456789abcdef0123456789abcdef";

fn success_body() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "event": "CONNECTION_SUCCESS",
        "connect_token": TOKEN,
        "environment": "development",
        "connect_session_id": 9007199254740993u64,
        "account": {
            "id": "apn_account",
            "name": "Calendar",
            "healthy": true,
            "future_account_field": {"preserve": true}
        },
        "future_event_field": [1, 2, 3]
    }))
    .unwrap()
}

#[test]
fn decodes_success_losslessly_and_keeps_unknown_fields() {
    let body = success_body();
    let event = ConnectionEvent::decode(&body).unwrap();
    assert_eq!(event.event, ConnectionEventKind::Success);
    assert_eq!(event.connect_token.expose_secret(), TOKEN);
    assert_eq!(
        event.connect_session_id.decode::<u64>().unwrap(),
        9007199254740993u64
    );
    assert_eq!(event.account.as_ref().unwrap().id.as_str(), "apn_account");
    assert_eq!(
        event.extra["future_event_field"]
            .decode::<Vec<u8>>()
            .unwrap(),
        vec![1, 2, 3]
    );
    assert_eq!(
        event.account.as_ref().unwrap().extra["future_account_field"],
        json!({"preserve": true})
    );
}

#[test]
fn preserves_session_numbers_larger_than_u64() {
    let body = br#"{
      "event":"CONNECTION_ERROR",
      "connect_token":"ctok_0123456789abcdef0123456789abcdef",
      "environment":"development",
      "connect_session_id":1.4058728083872854e+38,
      "error":"failed"
    }"#;
    let event = ConnectionEvent::decode(body).unwrap();
    assert_eq!(
        serde_json::to_string(&event.connect_session_id).unwrap(),
        "1.4058728083872854e+38"
    );
}

#[test]
fn decodes_error_payload_and_keeps_credentials_redacted() {
    let error = ConnectionEvent::decode(
        serde_json::to_vec(&json!({
            "event": "CONNECTION_ERROR",
            "connect_token": TOKEN,
            "environment": "production",
            "connect_session_id": "session-1",
            "error": "authorization failed",
            "future": "kept"
        }))
        .unwrap()
        .as_slice(),
    )
    .unwrap();
    assert_eq!(error.event, ConnectionEventKind::Error);
    assert_eq!(error.error.as_deref(), Some("authorization failed"));
    assert_eq!(error.extra["future"].decode::<String>().unwrap(), "kept");

    let body = serde_json::to_vec(&json!({
        "event": "CONNECTION_SUCCESS",
        "connect_token": TOKEN,
        "environment": "development",
        "connect_session_id": 7,
        "account": {
            "id": "apn_account",
            "credentials": {"access_token": "secret"}
        }
    }))
    .unwrap();
    let event = ConnectionEvent::decode(&body).unwrap();
    assert!(!format!("{event:?}").contains("secret"));
    assert!(event.account.unwrap().credentials.is_some());
}

#[test]
fn validates_shape_and_body_limit_before_json_processing() {
    assert!(matches!(
        ConnectionEvent::decode_with_limit(&[b'{'; 9], 8),
        Err(PayloadError::BodyTooLarge { limit: 8 })
    ));
    let body = serde_json::to_vec(&json!({
        "event": "CONNECTION_SUCCESS",
        "connect_token": TOKEN,
        "environment": "development",
        "connect_session_id": 7
    }))
    .unwrap();
    assert!(matches!(
        ConnectionEvent::decode(&body),
        Err(PayloadError::InvalidConnectionEvent)
    ));
    assert!(matches!(
        ConnectionEvent::decode(br#"{"event":"UNKNOWN"}"#),
        Err(PayloadError::InvalidJson)
    ));
}

#[test]
fn accepts_account_credentials_with_redacted_diagnostics() {
    let body = br#"{
        "event":"CONNECTION_SUCCESS",
        "connect_token":"ctok_0123456789abcdef0123456789abcdef",
        "environment":"development",
        "connect_session_id":7,
        "account":{"id":"apn_account","credentials":{"api_key":"credential-marker"}}
    }"#;
    let event = ConnectionEvent::decode(body).unwrap();
    assert!(!format!("{event:?}").contains("credential-marker"));
    assert_eq!(
        event.account.unwrap().credentials.unwrap().expose_secret()["api_key"],
        json!("credential-marker")
    );
}

#[test]
fn preserves_session_integer_above_u64_and_unknown_number_text() {
    let body = br#"{"event":"CONNECTION_ERROR","connect_token":"ctok_0123456789abcdef0123456789abcdef","environment":"development","connect_session_id":18446744073709551616,"error":"failed","future":123456789012345678901234567890}"#;
    let event = ConnectionEvent::decode(body).unwrap();
    assert_eq!(event.connect_session_id.as_str(), "18446744073709551616");
    assert_eq!(
        event.extra["future"].as_str(),
        "123456789012345678901234567890"
    );
    let encoded = serde_json::to_string(&event).unwrap();
    assert!(encoded.contains("18446744073709551616"));
    assert!(encoded.contains("123456789012345678901234567890"));
}
