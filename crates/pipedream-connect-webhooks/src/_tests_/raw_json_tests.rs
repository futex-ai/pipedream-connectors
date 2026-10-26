//! Exact JSON storage and source-preserving error tests.

use std::error::Error;

use crate::{FaultClass, PayloadError, RawJson};

#[test]
fn invalid_json_retains_the_serde_error_as_its_source() {
    let error = RawJson::new("{").unwrap_err();
    assert!(matches!(error, PayloadError::Json { .. }));
    assert_eq!(error.status_code(), 400);
    let source = error.source().unwrap();
    assert!(source.downcast_ref::<serde_json::Error>().unwrap().is_eof());
}

#[test]
fn typed_decode_failure_retains_the_serde_error_as_its_source() {
    let value = RawJson::new(r#""text""#).unwrap();
    let error = value.decode::<u64>().unwrap_err();
    assert!(matches!(error, PayloadError::Json { .. }));
    assert_eq!(error.status_code(), 400);
    let source = error.source().unwrap();
    assert!(
        source
            .downcast_ref::<serde_json::Error>()
            .unwrap()
            .is_data()
    );
}

#[test]
fn payload_status_codes_follow_fault_classes() {
    let invalid_json = RawJson::new("{").unwrap_err();
    let cases = [
        (
            PayloadError::BodyTooLarge { limit: 1 },
            FaultClass::Sender,
            413,
        ),
        (PayloadError::InvalidJson, FaultClass::Sender, 400),
        (invalid_json, FaultClass::Sender, 400),
        (
            PayloadError::InvalidConnectionEvent,
            FaultClass::Sender,
            400,
        ),
        (
            PayloadError::InvalidTriggerDelivery,
            FaultClass::Sender,
            400,
        ),
        (PayloadError::InvalidContextId, FaultClass::Sender, 400),
    ];
    for (error, class, status) in cases {
        assert_eq!(error.fault_class(), class);
        assert_eq!(error.status_code(), status);
    }
}
