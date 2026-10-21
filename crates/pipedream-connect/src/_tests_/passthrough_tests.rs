//! Proxy dispatch retains statuses, repeated headers, binary data, and 401 invalidation.

use std::num::NonZeroU32;
use std::time::Duration;

use http::{HeaderValue, Method};
use serde_json::json;

use crate::error::Error;
use crate::request::{ApiPath, ApiRequest, ResponseMode};
use crate::retry::RetryPolicy;

use super::support::{Harness, config, response, static_config, token};

#[tokio::test]
async fn passthrough_returns_every_status_header_and_byte_without_retry() {
    for status in [200, 301, 400, 401, 403, 404, 408, 429, 500] {
        let mut reply = response(status, json!({}));
        reply.body = vec![0, 255, 42];
        reply
            .headers
            .append("x-meta", HeaderValue::from_static("first"));
        reply
            .headers
            .append("x-meta", HeaderValue::from_static("second"));
        reply
            .headers
            .insert("x-binary", HeaderValue::from_bytes(&[255]).unwrap());
        let expected = reply.clone();
        let harness = Harness::script(
            static_config().with_retry_policy(RetryPolicy::new(
                NonZeroU32::new(3).unwrap(),
                Duration::from_secs(30),
                Duration::ZERO,
            )),
            vec![Ok(reply)],
        );
        let request = ApiRequest::new(Method::GET, ApiPath::project(&["proxy"]))
            .with_response_mode(ResponseMode::PassThrough);
        let actual = harness.client.dispatcher().dispatch(request).await.unwrap();
        assert_eq!(actual.status, expected.status);
        assert_eq!(actual.headers, expected.headers);
        assert_eq!(actual.body, expected.body);
        harness.assert_requests(1);
    }
}

#[tokio::test]
async fn passthrough_checks_execution_envelopes_before_returning_the_response() {
    let harness = Harness::script(
        static_config(),
        vec![Ok(response(
            400,
            json!({"error":{"attribution":{"origin":"upstream_api"},"message":"secret"}}),
        ))],
    );
    let request = ApiRequest::new(Method::GET, ApiPath::project(&["proxy"]))
        .with_response_mode(ResponseMode::PassThrough);
    assert!(matches!(
        harness.client.dispatcher().dispatch(request).await,
        Err(Error::ExecutionFailed(_))
    ));
    harness.assert_requests(1);
}

#[tokio::test]
async fn passthrough_invalidates_a_401_bearer_without_replay() {
    let harness = Harness::script(
        config(),
        vec![
            Ok(token("first", 3600)),
            Ok(response(401, json!({}))),
            Ok(token("second", 3600)),
            Ok(response(200, json!({}))),
        ],
    );
    let request = ApiRequest::new(Method::GET, ApiPath::project(&["proxy"]))
        .with_response_mode(ResponseMode::PassThrough);
    assert_eq!(
        harness
            .client
            .dispatcher()
            .dispatch(request)
            .await
            .unwrap()
            .status,
        401
    );
    harness.get().await.unwrap();
    assert_eq!(harness.request(3).headers["authorization"], "Bearer second");
    harness.assert_requests(4);
}
