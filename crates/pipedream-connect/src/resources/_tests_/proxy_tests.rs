//! Proxy target encoding, header admission, and lossless pass-through results.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use http::{HeaderMap, HeaderValue, StatusCode};
use pipedream_connect_types::{AccountId, AccountRef, ExternalUserId};
use serde_json::json;

use crate::error::Error;
use crate::resources::proxy::ProxyRequest;
use crate::tests::support::{Harness, response, static_config};

fn account() -> AccountRef {
    AccountRef::new(
        ExternalUserId::new("user_1").unwrap(),
        AccountId::new("apn_Account1").unwrap(),
    )
}

#[tokio::test]
async fn proxy_encodes_the_exact_target_prefixes_headers_and_preserves_bytes() {
    let mut result = response(404, json!({"provider":"value"}));
    result
        .headers
        .append("x-upstream", HeaderValue::from_static("one"));
    result
        .headers
        .append("x-upstream", HeaderValue::from_static("two"));
    result.body = vec![0, 1, 2, 255];
    let harness = Harness::script(static_config(), vec![Ok(result)]);
    let mut headers = HeaderMap::new();
    headers.insert("x-custom", HeaderValue::from_static("header"));
    let request = ProxyRequest::post(account(), "https://example.com/a%2Fb?query=a+b&limit=1")
        .with_headers(headers)
        .with_body(vec![9, 8, 7]);
    let response = harness.client.proxy().send(request).await.unwrap();
    assert_eq!(response.status, StatusCode::NOT_FOUND);
    assert_eq!(response.body, vec![0, 1, 2, 255]);
    assert_eq!(response.headers.get_all("x-upstream").iter().count(), 2);
    let sent = harness.request(0);
    let encoded = sent.url.expose_secret().split("/proxy/").nth(1).unwrap();
    let encoded = encoded.split('?').next().unwrap();
    assert_eq!(
        URL_SAFE_NO_PAD.decode(encoded).unwrap(),
        b"https://example.com/a%2Fb?query=a+b&limit=1"
    );
    assert_eq!(sent.method, http::Method::POST);
    assert_eq!(sent.headers["x-pd-proxy-x-custom"], "header");
    assert_eq!(sent.body, Some(vec![9, 8, 7]));
    assert!(!sent.headers.contains_key("content-type"));
    harness.assert_requests(1);
}

#[tokio::test]
async fn proxy_exposes_all_documented_methods() {
    let harness = Harness::script(
        static_config(),
        vec![
            Ok(response(200, json!({}))),
            Ok(response(200, json!({}))),
            Ok(response(200, json!({}))),
            Ok(response(200, json!({}))),
            Ok(response(200, json!({}))),
        ],
    );
    harness
        .client
        .proxy()
        .get(ProxyRequest::get(account(), "/resource"))
        .await
        .unwrap();
    harness
        .client
        .proxy()
        .post(ProxyRequest::post(account(), "/resource"))
        .await
        .unwrap();
    harness
        .client
        .proxy()
        .put(ProxyRequest::put(account(), "/resource"))
        .await
        .unwrap();
    harness
        .client
        .proxy()
        .patch(ProxyRequest::patch(account(), "/resource"))
        .await
        .unwrap();
    harness
        .client
        .proxy()
        .delete(ProxyRequest::delete(account(), "/resource"))
        .await
        .unwrap();
    for (index, method) in [
        http::Method::GET,
        http::Method::POST,
        http::Method::PUT,
        http::Method::PATCH,
        http::Method::DELETE,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(harness.request(index).method, method);
    }
    harness.assert_requests(5);
}

#[tokio::test]
async fn completed_upstream_error_statuses_remain_proxy_results() {
    for status in [401, 403, 404, 429, 500, 504] {
        let harness = Harness::script(
            static_config(),
            vec![Ok(response(status, json!({"upstream":true})))],
        );
        let result = harness
            .client
            .proxy()
            .get(ProxyRequest::get(account(), "/resource"))
            .await
            .unwrap();
        assert_eq!(result.status, StatusCode::from_u16(status).unwrap());
        assert_eq!(
            result.body,
            serde_json::to_vec(&json!({"upstream":true})).unwrap()
        );
        harness.assert_requests(1);
    }
}

#[tokio::test]
async fn every_restricted_proxy_header_is_rejected_before_authentication() {
    for name in [
        "Accept-Encoding",
        "Access-Control-Request-Headers",
        "Access-Control-Request-Method",
        "Connection",
        "Content-Length",
        "Cookie",
        "Date",
        "DNT",
        "Expect",
        "Host",
        "Keep-Alive",
        "Origin",
        "Permissions-Policy",
        "Referer",
        "TE",
        "Trailer",
        "Transfer-Encoding",
        "Upgrade",
        "Via",
        "Proxy-Authorization",
        "Sec-Fetch-Mode",
        "x-pd-environment",
    ] {
        let harness = Harness::script(static_config(), vec![]);
        let mut headers = HeaderMap::new();
        headers.insert(name, HeaderValue::from_static("blocked"));
        let result = harness
            .client
            .proxy()
            .send(ProxyRequest::get(account(), "/relative").with_headers(headers))
            .await;
        assert!(matches!(result, Err(Error::InvalidRequest(_))), "{name}");
        harness.assert_requests(0);
    }
}

#[tokio::test]
async fn invalid_proxy_targets_are_rejected_before_authentication() {
    for target in [
        "http://example.com/resource",
        "https://user:secret@example.com/resource",
        "https://example.com/resource#fragment",
        "//example.com/resource",
    ] {
        let harness = Harness::script(static_config(), vec![]);
        let result = harness
            .client
            .proxy()
            .get(ProxyRequest::get(account(), target))
            .await;
        assert!(matches!(result, Err(Error::InvalidRequest(_))), "{target}");
        harness.assert_requests(0);
    }
}
