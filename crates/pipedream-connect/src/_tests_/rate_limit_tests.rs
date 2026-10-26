//! Custom rate-limit reset parsing.

use std::time::Duration;

use http::HeaderValue;
use serde_json::json;

use crate::error::Error;

use super::support::{Harness, response, static_config};

#[tokio::test]
async fn custom_rate_limit_reset_delay_is_optional_and_bounded_to_digits() {
    for (header, expected) in [
        ("17", Some(17)),
        ("0", Some(0)),
        ("-1", None),
        ("garbage", None),
        ("18446744073709551616", None),
    ] {
        let mut reply = response(429, json!({}));
        reply
            .headers
            .insert("x-pd-rate-limit-reset-in", HeaderValue::from_static(header));
        let harness = Harness::script(static_config(), vec![Ok(reply)]);
        assert!(matches!(
            harness.get().await,
            Err(Error::RateLimited {
                reset_after,
                ..
            }) if reset_after == expected.map(Duration::from_secs)
        ));
        harness.assert_requests(1);
    }
}
