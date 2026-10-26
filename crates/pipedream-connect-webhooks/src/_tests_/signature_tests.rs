//! Fixed-vector tests for raw-body signature admission.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use pipedream_connect_types::SecretString;

use crate::{
    DEFAULT_BODY_LIMIT_BYTES, DEFAULT_SIGNATURE_TOLERANCE_SECONDS, FaultClass,
    HmacWebhookSignatureVerifier, SignatureError, WebhookSignatureVerifier,
};

const KEY: &str = "webhook-test-key";
const HEADER: &str = "t=1000,v1=6e77b54efa731c70ca5c7004572d6ac24203eebfd2dc3b4981b82de56d187ea5";
const BODY: &[u8] = br#"{"event":"created"}
"#;

fn receipt(seconds: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

fn verifier() -> HmacWebhookSignatureVerifier {
    HmacWebhookSignatureVerifier::new().with_body_limit(1024)
}

#[test]
fn verifier_defaults_to_ten_megabytes_and_accepts_a_custom_limit() {
    assert_eq!(
        HmacWebhookSignatureVerifier::new().body_limit(),
        DEFAULT_BODY_LIMIT_BYTES
    );
    assert_eq!(
        HmacWebhookSignatureVerifier::default().body_limit(),
        DEFAULT_BODY_LIMIT_BYTES
    );
    assert_eq!(verifier().body_limit(), 1024);
}

#[test]
fn accepts_valid_signatures_at_both_default_boundaries() {
    assert_eq!(
        verifier().tolerance_seconds(),
        DEFAULT_SIGNATURE_TOLERANCE_SECONDS
    );
    for received_at in [700, 1000, 1300] {
        assert_eq!(
            verifier().verify(&SecretString::new(KEY), HEADER, BODY, receipt(received_at),),
            Ok(())
        );
    }
}

#[test]
fn rejects_stale_and_future_timestamps() {
    for received_at in [699, 1301] {
        assert_eq!(
            verifier().verify(&SecretString::new(KEY), HEADER, BODY, receipt(received_at),),
            Err(SignatureError::StaleTimestamp)
        );
    }
}

#[test]
fn rejects_duplicate_parts_unknown_parts_bad_hex_and_wrong_lengths() {
    let digest = "a".repeat(64);
    for header in [
        format!("t=1000,v1={digest},t=1000"),
        format!("t=1000,v1={digest},v1={digest}"),
        format!("t=1000,v2={digest}"),
        format!("v1={digest},t=1000"),
        format!("t=1000,v1={}", "a".repeat(63)),
        format!("t=1000,v1={}z", "a".repeat(63)),
        "t=+1000,v1=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
        "t=,v1=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
        "t=1000".to_owned(),
        "v1=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
        "t=1000,v1=".to_owned(),
        "t=1000,v1=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa,".to_owned(),
    ] {
        assert_eq!(
            verifier().verify(&SecretString::new(KEY), &header, BODY, receipt(1000),),
            Err(SignatureError::InvalidHeader)
        );
    }
}

#[test]
fn status_codes_follow_sender_and_host_fault_classes() {
    let cases = [
        (SignatureError::InvalidHeader, FaultClass::Sender, 400),
        (SignatureError::InvalidKey, FaultClass::Host, 500),
        (SignatureError::StaleTimestamp, FaultClass::Sender, 401),
        (SignatureError::InvalidSignature, FaultClass::Sender, 401),
        (SignatureError::InvalidReceiptTime, FaultClass::Host, 500),
        (
            SignatureError::BodyTooLarge { limit: 1 },
            FaultClass::Sender,
            413,
        ),
    ];
    for (error, class, status) in cases {
        assert_eq!(error.fault_class(), class);
        assert_eq!(error.status_code(), status);
    }
}

#[test]
fn rejects_empty_keys_and_oversized_bodies() {
    assert_eq!(
        verifier().verify(&SecretString::new(""), HEADER, BODY, receipt(1000),),
        Err(SignatureError::InvalidKey)
    );
    assert_eq!(
        verifier().verify(&SecretString::new(KEY), HEADER, &[0; 1025], receipt(1000),),
        Err(SignatureError::BodyTooLarge { limit: 1024 })
    );
}

#[test]
fn uses_the_exact_body_and_key_and_supports_custom_tolerance() {
    assert_eq!(
        verifier().verify(&SecretString::new("wrong-key"), HEADER, BODY, receipt(1000),),
        Err(SignatureError::InvalidSignature)
    );
    assert_eq!(
        verifier().verify(
            &SecretString::new(KEY),
            HEADER,
            br#"{"event":"created"}"#,
            receipt(1000),
        ),
        Err(SignatureError::InvalidSignature)
    );
    let narrow = verifier().with_tolerance_seconds(1);
    assert_eq!(
        narrow.verify(&SecretString::new(KEY), HEADER, BODY, receipt(1002),),
        Err(SignatureError::StaleTimestamp)
    );
}

#[test]
fn receipt_time_before_epoch_is_typed_without_exposing_secrets() {
    let error = verifier()
        .verify(
            &SecretString::new(KEY),
            HEADER,
            BODY,
            UNIX_EPOCH - Duration::from_secs(1),
        )
        .unwrap_err();
    assert_eq!(error, SignatureError::InvalidReceiptTime);
    let text = error.to_string();
    assert!(!text.contains(KEY));
    assert!(!text.contains(HEADER));
    assert!(!text.contains(std::str::from_utf8(BODY).unwrap()));
}
