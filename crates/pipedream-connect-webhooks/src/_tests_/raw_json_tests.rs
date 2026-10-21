//! Exact JSON storage and source-preserving error tests.

use std::error::Error;

use crate::{PayloadError, RawJson};

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
