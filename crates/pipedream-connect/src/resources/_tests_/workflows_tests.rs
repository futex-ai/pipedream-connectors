//! Workflow endpoint validation and pass-through tests.

use http::{HeaderValue, Method, StatusCode};
use pipedream_connect_types::{ExternalUserId, SecretString, WorkflowRequest};
use serde_json::json;

use crate::error::Error;
use crate::resources::test_support::assert_wire;
use crate::tests::support::{Harness, response, static_config};

#[tokio::test]
async fn workflow_invocation_sends_endpoint_headers_and_bytes() {
    let mut reply = response(201, json!({"result":"ok"}));
    reply
        .headers
        .insert("x-upstream", HeaderValue::from_static("value"));
    let harness = Harness::script(
        static_config()
            .with_rate_limit_token(SecretString::new("rate-secret"))
            .unwrap(),
        vec![Ok(reply)],
    );
    let result = harness
        .client
        .workflows()
        .invoke_for_external_user(
            "https://workflow.example/endpoint?one=1".into(),
            ExternalUserId::new("user_1").unwrap(),
            Method::POST,
            WorkflowRequest {
                body: Some(b"{\"message\":\"hello\"}".to_vec()),
                content_type: Some("application/json".into()),
            },
        )
        .await
        .unwrap();
    assert_eq!(result.status, StatusCode::CREATED);
    let request = harness.request(0);
    assert_wire(
        &request,
        Method::POST,
        "https://workflow.example/endpoint?one=1",
        &[
            ("authorization", "Bearer static-secret"),
            ("x-pd-external-user-id", "user_1"),
            ("x-pd-environment", "development"),
            ("content-type", "application/json"),
        ],
        Some(&json!({"message":"hello"})),
    );
    assert!(!request.headers.contains_key("x-pd-rate-limit"));
    harness.assert_requests(1);
}

#[tokio::test]
async fn workflow_pass_through_preserves_non_success_attribution_body() {
    let body = json!({
        "error": {
            "attribution": {
                "origin": "upstream_api",
                "network_calls_count": 1,
                "last_call": {"status": 502}
            }
        }
    });
    let harness = Harness::new(vec![response(502, body.clone())]);
    let result = harness
        .client
        .workflows()
        .invoke_for_external_user(
            "https://workflow.example/endpoint".into(),
            ExternalUserId::new("user_1").unwrap(),
            Method::POST,
            WorkflowRequest::default(),
        )
        .await
        .unwrap();

    assert_eq!(result.status, StatusCode::BAD_GATEWAY);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&result.body).unwrap(),
        body
    );
    assert_wire(
        &harness.request(1),
        Method::POST,
        "https://workflow.example/endpoint",
        &[
            ("authorization", "Bearer access-secret"),
            ("x-pd-environment", "development"),
            ("x-pd-external-user-id", "user_1"),
        ],
        None,
    );
    harness.assert_requests(2);
}

#[tokio::test]
async fn workflow_endpoint_is_validated_before_token_acquisition() {
    for endpoint in [
        "http://workflow.example/endpoint",
        "https://user:pass@workflow.example/endpoint",
        "https://workflow.example/endpoint#fragment",
        "https://workflow.example/endpoint\n",
    ] {
        let harness = Harness::script(static_config(), vec![]);
        let error = harness
            .client
            .workflows()
            .invoke_for_external_user(
                endpoint.into(),
                ExternalUserId::new("user_1").unwrap(),
                Method::POST,
                WorkflowRequest::default(),
            )
            .await
            .unwrap_err();
        assert!(matches!(error, Error::InvalidRequest(_)));
        harness.assert_requests(0);
    }
}

#[tokio::test]
async fn workflow_rejects_methods_outside_the_documented_set() {
    let harness = Harness::script(static_config(), vec![]);
    let error = harness
        .client
        .workflows()
        .invoke_for_external_user(
            "https://workflow.example/endpoint".into(),
            ExternalUserId::new("user_1").unwrap(),
            Method::HEAD,
            WorkflowRequest::default(),
        )
        .await
        .unwrap_err();
    assert!(matches!(error, Error::InvalidRequest(_)));
    harness.assert_requests(0);
}
