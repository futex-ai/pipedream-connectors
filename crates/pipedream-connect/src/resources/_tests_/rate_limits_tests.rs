//! Custom rate-limit response tests.

use pipedream_connect_types::RateLimitRequest;
use serde_json::json;

use crate::resources::test_support::{ExpectedBody, assert_wire};
use crate::tests::support::{Harness, response, static_config};

#[tokio::test]
async fn rate_limit_token_is_wrapped_and_request_body_matches_json() {
    let harness = Harness::script(
        static_config(),
        vec![
            Ok(response(200, json!({"token":"secret"}))),
            Ok(response(200, json!({"data":[],"page_info":{"count":0}}))),
        ],
    );
    let token = harness
        .client
        .rate_limits()
        .create(RateLimitRequest {
            window_size_seconds: Some(5),
            requests_per_window: Some(10),
        })
        .await
        .unwrap();
    assert_eq!(token.token.expose_secret(), "secret");
    assert!(!format!("{token:?}").contains("secret"));
    assert_wire(
        &harness.request(0),
        http::Method::POST,
        "https://api.pipedream.com/v1/connect/rate_limits",
        &[
            ("authorization", "Bearer static-secret"),
            ("x-pd-environment", "development"),
            ("accept", "application/json"),
            ("content-type", "application/json"),
        ],
        Some(ExpectedBody::Json(
            &json!({"window_size_seconds":5,"requests_per_window":10}),
        )),
    );
    let configured = harness
        .client
        .clone()
        .with_rate_limit_token(token.token)
        .unwrap();
    configured
        .projects()
        .list(pipedream_connect_types::ListProjectsRequest::default())
        .await
        .unwrap();
    assert_eq!(harness.request(1).headers["x-pd-rate-limit"], "secret");
    harness.assert_requests(2);
}
