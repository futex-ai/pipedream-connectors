//! Fixed payload tests for trigger event identity and context preservation.

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::{PayloadError, TriggerDelivery};

#[test]
fn uses_context_id_and_preserves_event_and_unknown_context_data() {
    let body = br#"{
      "event":{"amount":9007199254740993,"nested":{"value":true}},
      "context":{"id":"delivery-1","ts":"2026-01-01T00:00:00Z","future":42},
      "future_delivery_field":"kept"
    }"#;
    let delivery = TriggerDelivery::decode(body).unwrap();
    assert_eq!(delivery.identity(), "delivery-1");
    assert_eq!(delivery.identity(), "delivery-1");
    assert_eq!(
        delivery.context.timestamp.unwrap().to_rfc3339(),
        "2026-01-01T00:00:00+00:00"
    );
    assert_eq!(
        delivery.context.extra["future"].decode::<u32>().unwrap(),
        42
    );
    assert_eq!(
        delivery.extra["future_delivery_field"]
            .decode::<String>()
            .unwrap(),
        "kept"
    );
    assert_eq!(
        delivery.event.decode::<serde_json::Value>().unwrap()["amount"],
        json!(9007199254740993u64)
    );
}

#[test]
fn preserves_large_numbers_in_app_event_json() {
    let body = br#"{"event":{"value":123456789012345678901234567890}}"#;
    let delivery = TriggerDelivery::decode(body).unwrap();
    assert_eq!(
        serde_json::to_string(&delivery.event).unwrap(),
        r#"{"value":123456789012345678901234567890}"#
    );
}

#[test]
fn falls_back_to_the_sha256_digest_of_exact_raw_body() {
    let body = br#"{"event":{"created":true}}"#;
    let delivery = TriggerDelivery::decode(body).unwrap();
    assert_eq!(delivery.identity(), hex::encode(Sha256::digest(body)));
    assert_ne!(
        delivery.identity(),
        TriggerDelivery::decode(br#"{"event":{"created":false}}"#)
            .unwrap()
            .identity()
    );
}

#[test]
fn rejects_invalid_context_ids_and_oversized_bodies() {
    for id in ["", "line\nbreak", &"x".repeat(251)] {
        let body = serde_json::to_vec(&json!({
            "event": {},
            "context": {"id": id}
        }))
        .unwrap();
        assert!(matches!(
            TriggerDelivery::decode(&body),
            Err(PayloadError::InvalidContextId)
        ));
    }
    assert!(matches!(
        TriggerDelivery::decode_with_limit(&[b'{'; 9], 8),
        Err(PayloadError::BodyTooLarge { limit: 8 })
    ));
    assert!(matches!(
        TriggerDelivery::decode(br#"{"context":{}}"#),
        Err(PayloadError::InvalidTriggerDelivery)
    ));
}

#[test]
fn exact_event_text_survives_typed_access_and_reencoding() {
    let body = br#"{"event": {"fraction":1.234567890123456789,"integer":9007199254740993},"context":{"future":18446744073709551616},"future":9007199254740993}"#;
    let delivery = TriggerDelivery::decode(body).unwrap();
    assert_eq!(
        delivery.event.as_str(),
        r#"{"fraction":1.234567890123456789,"integer":9007199254740993}"#
    );
    assert_eq!(
        delivery.context.extra["future"].as_str(),
        "18446744073709551616"
    );
    assert!(
        serde_json::to_string(&delivery)
            .unwrap()
            .contains("1.234567890123456789")
    );
}

#[test]
fn serde_decoding_initializes_the_private_delivery_identity() {
    for body in [
        &br#"{"event":{"x":1},"context":{"id":"delivery-1"}}"#[..],
        &br#"{ "event" : {"x":1} }"#[..],
    ] {
        let decoded: TriggerDelivery = serde_json::from_slice(body).unwrap();
        let checked = TriggerDelivery::decode(body).unwrap();
        assert_eq!(decoded.identity(), checked.identity());
    }
}

#[test]
fn hashes_trailing_newline_bytes_in_both_decode_methods() {
    let body = b"{\"event\":\"created\"}\n";
    for delivery in [
        TriggerDelivery::decode(body).unwrap(),
        TriggerDelivery::decode_with_limit(body, body.len()).unwrap(),
    ] {
        assert_eq!(delivery.identity(), hex::encode(Sha256::digest(body)));
    }
}

#[test]
fn hashes_leading_whitespace_bytes_in_both_decode_methods() {
    let body = b" \t\n{\"event\":\"created\"}";
    for delivery in [
        TriggerDelivery::decode(body).unwrap(),
        TriggerDelivery::decode_with_limit(body, body.len()).unwrap(),
    ] {
        assert_eq!(delivery.identity(), hex::encode(Sha256::digest(body)));
    }
}

#[test]
fn serde_rejects_invalid_context_ids() {
    for id in ["", "line\nbreak", "control\u{7f}", &"x".repeat(251)] {
        let body = serde_json::to_vec(&json!({"event": {}, "context": {"id": id}})).unwrap();
        assert!(serde_json::from_slice::<TriggerDelivery>(&body).is_err());
    }
}

#[test]
fn invalid_delivery_shapes_return_the_delivery_error() {
    for body in [
        &br#"{"context":{}}"#[..],
        &br#"[]"#[..],
        &br#"{"event":{},"context":false}"#[..],
    ] {
        assert!(matches!(
            TriggerDelivery::decode(body),
            Err(PayloadError::InvalidTriggerDelivery)
        ));
    }
}

#[test]
fn serde_hashes_the_value_text_without_outer_whitespace() {
    let body = b" \t{\"event\": {\"created\": true}}\n";
    let value_text = br#"{"event": {"created": true}}"#;
    let delivery: TriggerDelivery = serde_json::from_slice(body).unwrap();
    assert_eq!(delivery.identity(), hex::encode(Sha256::digest(value_text)));
    assert_ne!(
        delivery.identity(),
        TriggerDelivery::decode(body).unwrap().identity()
    );
}
