//! Shared route encoding, HTTP headers, bounds, and diagnostic safety.

use bytes::Bytes;
use http::{HeaderMap, HeaderValue, Method};
use pipedream_connect_types::{
    CreateConnectTokenRequest, Environment, ExternalUserId, OAuthTokenResponse, ProjectId,
    SecretString,
};
use serde_json::{Value, json};
use url::Url;

use crate::config::{Authentication, ClientConfig, HttpOptions};
use crate::error::{Error, InvalidRequest};
use crate::request::{ApiPath, ApiRequest, ResponseMode};

use super::support::{Harness, config, response, static_config};

#[tokio::test]
async fn oauth_and_connect_headers_bodies_and_scope_are_exact() {
    let config = config()
        .with_scope("read write")
        .with_rate_limit_token(SecretString::new("rate-secret"))
        .unwrap();
    let harness = Harness::script(
        config,
        vec![
            Ok(super::support::token("access-secret", 3600)),
            Ok(response(200, json!({}))),
        ],
    );
    let model = CreateConnectTokenRequest::new(ExternalUserId::new("user_1").unwrap());
    let request = ApiRequest::new(Method::POST, ApiPath::project(&["tokens"]))
        .with_json(&model)
        .unwrap();
    harness.client.dispatcher().dispatch(request).await.unwrap();
    let oauth = harness.request(0);
    assert_eq!(
        oauth.url.expose_secret(),
        "https://api.pipedream.com/v1/oauth/token"
    );
    assert_eq!(
        serde_json::from_slice::<Value>(oauth.body.as_ref().unwrap()).unwrap(),
        json!({"grant_type":"client_credentials","client_id":"client-id-secret","client_secret":"client-secret","scope":"read write"})
    );
    for name in ["authorization", "x-pd-environment", "x-pd-rate-limit"] {
        assert!(!oauth.headers.contains_key(name));
    }
    let connect = harness.request(1);
    assert_eq!(
        connect.url.expose_secret(),
        "https://api.pipedream.com/v1/connect/proj_Test123/tokens"
    );
    assert_eq!(connect.headers["authorization"], "Bearer access-secret");
    assert_eq!(connect.headers["x-pd-environment"], "development");
    assert_eq!(connect.headers["x-pd-rate-limit"], "rate-secret");
    for request in [&oauth, &connect] {
        assert_eq!(request.headers["accept"], "application/json");
        assert_eq!(request.headers["content-type"], "application/json");
        assert_eq!(
            request.headers["user-agent"],
            concat!("pipedream-connect/", env!("CARGO_PKG_VERSION"))
        );
        assert!(!format!("{request:?}").contains("secret"));
    }
    harness.assert_requests(2);
}

#[tokio::test]
async fn static_bearer_uses_no_oauth_and_sdk_headers_replace_caller_values() {
    let config = ClientConfig::new(
        ProjectId::new("proj_Test123").unwrap(),
        Environment::Production,
        Authentication::StaticBearer(SecretString::new("static-secret")),
    )
    .unwrap();
    let harness = Harness::script(config, vec![Ok(response(200, json!({})))]);
    let mut headers = HeaderMap::new();
    headers.insert("authorization", HeaderValue::from_static("wrong"));
    headers.insert("x-pd-environment", HeaderValue::from_static("wrong"));
    headers.insert("x-pd-rate-limit", HeaderValue::from_static("wrong"));
    let request = ApiRequest::new(Method::GET, ApiPath::connect(&["apps"]))
        .with_headers(headers)
        .query_pair("q", "a&b #c")
        .query_pair("category_ids", "one")
        .query_pair("category_ids", "two");
    harness.client.dispatcher().dispatch(request).await.unwrap();
    let request = harness.request(0);
    assert_eq!(request.headers["authorization"], "Bearer static-secret");
    assert_eq!(request.headers["x-pd-environment"], "production");
    assert!(!request.headers.contains_key("x-pd-rate-limit"));
    assert!(!request.headers.contains_key("content-type"));
    let url = Url::parse(request.url.expose_secret()).unwrap();
    assert_eq!(url.path(), "/v1/connect/apps");
    assert_eq!(
        url.query_pairs().into_owned().collect::<Vec<_>>(),
        [
            ("q".into(), "a&b #c".into()),
            ("category_ids".into(), "one".into()),
            ("category_ids".into(), "two".into())
        ]
    );
    harness.assert_requests(1);
}

#[tokio::test]
async fn repeated_with_headers_calls_merge_and_replace_only_repeated_names() {
    let harness = Harness::script(static_config(), vec![Ok(response(200, json!({})))]);
    let mut first = HeaderMap::new();
    first.insert("x-first", HeaderValue::from_static("one"));
    first.insert("x-overlap", HeaderValue::from_static("old"));
    let mut second = HeaderMap::new();
    second.insert("x-second", HeaderValue::from_static("two"));
    second.insert("x-overlap", HeaderValue::from_static("new"));
    let request = ApiRequest::new(Method::GET, ApiPath::connect(&["apps"]))
        .with_headers(first)
        .with_headers(second);
    harness.client.dispatcher().dispatch(request).await.unwrap();
    let sent = harness.request(0);
    assert_eq!(sent.headers["x-first"], "one");
    assert_eq!(sent.headers["x-second"], "two");
    assert_eq!(sent.headers["x-overlap"], "new");
    harness.assert_requests(1);
}

#[tokio::test]
async fn absolute_target_validation_reports_a_workflow_endpoint_error() {
    let error = ApiPath::absolute("not a URL").unwrap_err();
    assert!(matches!(
        error,
        Error::InvalidRequest(InvalidRequest::WorkflowEndpoint)
    ));

    let harness = Harness::script(static_config(), vec![]);
    let path = ApiPath::Absolute(Url::parse("https://user:pass@example.com/path").unwrap());
    let error = harness
        .client
        .dispatcher()
        .dispatch(ApiRequest::new(Method::GET, path))
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        Error::InvalidRequest(InvalidRequest::WorkflowEndpoint)
    ));
    harness.assert_requests(0);
}

#[tokio::test]
async fn path_parameters_cannot_inject_routes_and_dot_segments_are_skipped() {
    let harness = Harness::script(
        static_config(),
        (0..5).map(|_| Ok(response(200, json!({})))).collect(),
    );
    for (segment, expected) in [
        ("a/b", "a%2Fb"),
        ("a%b", "a%25b"),
        ("a?b#c", "a%3Fb%23c"),
        (".", ""),
        ("..", ""),
    ] {
        let request = ApiRequest::new(
            Method::GET,
            ApiPath::project(&["users", segment, "accounts"]),
        );
        harness.client.dispatcher().dispatch(request).await.unwrap();
        let recorded = harness.requests.lock().unwrap().last().unwrap().clone();
        let url = Url::parse(recorded.url.expose_secret()).unwrap();
        let expected = if expected.is_empty() {
            "/v1/connect/proj_Test123/users/accounts".to_owned()
        } else {
            format!("/v1/connect/proj_Test123/users/{expected}/accounts")
        };
        assert_eq!(url.path(), expected);
        assert!(url.query().is_none());
        assert!(url.fragment().is_none());
    }
    harness.assert_requests(5);
}

#[tokio::test]
async fn injected_transport_bodies_and_announced_sizes_are_bounded() {
    for announced in [false, true] {
        let config = static_config().with_http_options(HttpOptions {
            response_limit: 2,
            ..HttpOptions::default()
        });
        let mut reply = response(200, json!({}));
        if announced {
            reply
                .headers
                .insert("content-length", HeaderValue::from_static("3"));
        } else {
            reply.body.push(b' ');
        }
        let harness = Harness::script(config, vec![Ok(reply)]);
        assert!(matches!(
            harness.get().await,
            Err(Error::ResponseTooLarge {
                status: 200,
                limit: 2
            })
        ));
        assert_eq!(harness.request(0).response_limit, 2);
        harness.assert_requests(1);
    }
}

#[tokio::test]
async fn success_decode_error_exposes_only_location_and_no_serde_message() {
    let reply = response(
        200,
        json!({"access_token":"sensitive-secret","token_type":"bearer","expires_in":"secret"}),
    );
    let diagnostic = format!("{reply:?}");
    assert!(!diagnostic.contains("sensitive-secret"));
    let harness = Harness::new(vec![reply]);
    let reply = harness.get().await.unwrap();
    let error = reply.json::<OAuthTokenResponse>().unwrap_err();
    assert!(matches!(error, Error::Internal(_)));
    let diagnostic = format!("{error:?} {error}");
    assert!(!diagnostic.contains("secret"));
    harness.assert_requests(2);
}

#[test]
fn local_model_errors_cannot_expose_internal_decoder_values() {
    let source = serde_json::from_str::<u32>("\"sensitive-secret\"").unwrap_err();
    let model = pipedream_connect_types::Error::internal(source);
    let error = Error::InvalidRequest(InvalidRequest::Model(model));
    assert!(!format!("{error:?} {error}").contains("sensitive-secret"));
}

#[tokio::test]
async fn response_mode_does_not_change_request_content_type() {
    let harness = Harness::script(
        static_config(),
        vec![Ok(response(200, json!({}))), Ok(response(200, json!({})))],
    );
    let model = CreateConnectTokenRequest::new(ExternalUserId::new("user_1").unwrap());
    let request = ApiRequest::new(Method::POST, ApiPath::project(&["tokens"]))
        .with_json(&model)
        .unwrap()
        .with_response_mode(ResponseMode::Bytes);
    harness.client.dispatcher().dispatch(request).await.unwrap();
    assert_eq!(
        harness.request(0).headers["content-type"],
        "application/json"
    );
    assert!(!harness.request(0).headers.contains_key("accept"));
    let mut headers = HeaderMap::new();
    headers.insert(
        "content-type",
        HeaderValue::from_static("application/octet-stream"),
    );
    let request = ApiRequest::new(Method::POST, ApiPath::project(&["files"]))
        .with_headers(headers)
        .with_bytes(vec![0, 255, 12]);
    harness.client.dispatcher().dispatch(request).await.unwrap();
    assert_eq!(
        harness.request(1).headers["content-type"],
        "application/octet-stream"
    );
    assert_eq!(
        harness.request(1).body.as_deref(),
        Some([0, 255, 12].as_slice())
    );
    harness.assert_requests(2);
}

#[tokio::test]
async fn request_bodies_use_shared_bytes_buffers() {
    let body = Bytes::from_static(&[0, 255, 12]);
    let harness = Harness::script(static_config(), vec![Ok(response(200, json!({})))]);
    let request =
        ApiRequest::new(Method::POST, ApiPath::project(&["files"])).with_bytes(body.clone());

    harness.client.dispatcher().dispatch(request).await.unwrap();

    assert_eq!(harness.request(0).body, Some(body));
    harness.assert_requests(1);
}
