//! Sharing refresh failures preserves custom-transport validation categories.

use std::sync::Arc;

use pipedream_connect_types::{self, ValidationField};

use crate::error::{Error, InvalidRequest};
use crate::token_failure::repeat_failure;

#[test]
fn shared_failures_keep_local_validation_errors_typed() {
    let original = Arc::new(Error::InvalidRequest(InvalidRequest::Origin));
    assert!(matches!(
        repeat_failure(&original),
        Error::InvalidRequest(InvalidRequest::Origin)
    ));
    let original = Arc::new(Error::InvalidRequest(InvalidRequest::Model(
        pipedream_connect_types::Error::OutOfRange {
            field: ValidationField::ExpiresIn,
            minimum: 1,
            maximum: 14_400,
        },
    )));
    assert!(matches!(
        repeat_failure(&original),
        Error::InvalidRequest(InvalidRequest::Model(
            pipedream_connect_types::Error::OutOfRange {
                field: ValidationField::ExpiresIn,
                minimum: 1,
                maximum: 14_400
            }
        ))
    ));
}
