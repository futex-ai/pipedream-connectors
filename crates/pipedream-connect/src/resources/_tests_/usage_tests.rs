//! Usage and custom rate-limit request tests.

use pipedream_connect_types::{ListUsageRequest, RateLimitRequest};
use serde_json::json;

use crate::resources::test_support::{ExpectedBody, assert_wire};
use crate::tests::support::{Harness, response, static_config};

#[tokio::test]
async fn usage_query_and_rate_limit_body_use_typed_models() {
    let harness = Harness::script(
        static_config(),
        vec![
            Ok(response(
                200,
                json!({"data":[{"credits_used":3,"usage_start_ts":1,"usage_end_ts":2}]}),
            )),
            Ok(response(200, json!({"token":"rate-secret"}))),
        ],
    );
    let usage = harness
        .client
        .usage()
        .list(ListUsageRequest {
            start_ts: 1,
            end_ts: 2,
        })
        .await
        .unwrap();
    assert_eq!(usage.data[0].credits_used, 3);
    assert_wire(
        &harness.request(0),
        http::Method::GET,
        "https://api.pipedream.com/v1/connect/usage?end_ts=2&start_ts=1",
        &[
            ("authorization", "Bearer static-secret"),
            ("x-pd-environment", "development"),
            ("accept", "application/json"),
        ],
        None,
    );
    let token = harness
        .client
        .rate_limits()
        .create(RateLimitRequest {
            window_size_seconds: Some(10),
            requests_per_window: Some(100),
        })
        .await
        .unwrap();
    assert_eq!(token.token.expose_secret(), "rate-secret");
    assert!(!format!("{:?}", token).contains("rate-secret"));
    assert_wire(
        &harness.request(1),
        http::Method::POST,
        "https://api.pipedream.com/v1/connect/rate_limits",
        &[
            ("authorization", "Bearer static-secret"),
            ("x-pd-environment", "development"),
            ("accept", "application/json"),
            ("content-type", "application/json"),
        ],
        Some(ExpectedBody::Json(
            &json!({"window_size_seconds":10,"requests_per_window":100}),
        )),
    );
    harness.assert_requests(2);
}
