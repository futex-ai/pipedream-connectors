//! Workflow endpoint validation and pass-through tests.

use http::{HeaderValue, Method, StatusCode};
use pipedream_connect_types::{ExternalUserId, SecretString, WorkflowRequest};
use serde_json::json;

use crate::error::Error;
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
    assert_eq!(
        request.url.expose_secret(),
        "https://workflow.example/endpoint?one=1"
    );
    assert_eq!(request.headers["x-pd-external-user-id"], "user_1");
    assert_eq!(request.headers["x-pd-environment"], "development");
    assert!(!request.headers.contains_key("x-pd-rate-limit"));
    assert_eq!(request.headers["content-type"], "application/json");
    assert_eq!(
        request.body.as_deref(),
        Some(b"{\"message\":\"hello\"}".as_slice())
    );
    harness.assert_requests(1);
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
