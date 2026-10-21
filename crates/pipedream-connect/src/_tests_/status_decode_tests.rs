//! Success mapping does no error-envelope decode work.

use std::time::UNIX_EPOCH;

use http::{HeaderMap, StatusCode};

use crate::request::ResponseMode;
use crate::transport::HttpResponse;

use super::classify;

#[test]
fn successful_responses_never_decode_the_error_envelope() {
    let response = HttpResponse {
        status: StatusCode::OK,
        headers: HeaderMap::new(),
        body: b"{}".to_vec(),
    };
    for mode in [
        ResponseMode::Json,
        ResponseMode::Bytes,
        ResponseMode::Redirect,
    ] {
        classify(&response, mode, UNIX_EPOCH, |_| {
            panic!("success decoded as an error envelope")
        })
        .unwrap();
    }
}
