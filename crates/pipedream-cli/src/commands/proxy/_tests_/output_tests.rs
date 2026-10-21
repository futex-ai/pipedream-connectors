//! Tests for the lossless JSON form of an upstream proxy response.

use http::{HeaderMap, HeaderValue, StatusCode};
use pipedream_connect::transport::HttpResponse;
use serde_json::json;

use crate::commands::proxy::output::ProxyOutput;

fn output(headers: HeaderMap, body: &[u8]) -> serde_json::Value {
    let response = HttpResponse {
        status: StatusCode::NOT_FOUND,
        headers,
        body: body.to_vec(),
    };
    serde_json::to_value(ProxyOutput::from(response)).unwrap()
}

#[test]
fn text_bodies_and_headers_print_as_utf8_and_keep_repeated_names() {
    let mut headers = HeaderMap::new();
    headers.append("content-type", HeaderValue::from_static("application/json"));
    headers.append("set-cookie", HeaderValue::from_static("a=1"));
    headers.append("set-cookie", HeaderValue::from_static("b=2"));
    assert_eq!(
        output(headers, br#"{"ok":false}"#),
        json!({
            "status": 404,
            "headers": [
                {"name": "content-type", "encoding": "utf-8", "value": "application/json"},
                {"name": "set-cookie", "encoding": "utf-8", "value": "a=1"},
                {"name": "set-cookie", "encoding": "utf-8", "value": "b=2"},
            ],
            "body": {"encoding": "utf-8", "value": "{\"ok\":false}"},
        })
    );
}

#[test]
fn bytes_that_are_not_utf8_print_as_base64() {
    let mut headers = HeaderMap::new();
    headers.append("x-raw", HeaderValue::from_bytes(&[0xff, 0x41]).unwrap());
    let printed = output(headers, &[0x89, b'P', b'N', b'G']);
    assert_eq!(
        printed["headers"],
        json!([{"name": "x-raw", "encoding": "base64", "value": "/0E="}])
    );
    assert_eq!(
        printed["body"],
        json!({"encoding": "base64", "value": "iVBORw=="})
    );
}

#[test]
fn an_empty_body_prints_as_empty_text() {
    assert_eq!(
        output(HeaderMap::new(), b"")["body"],
        json!({"encoding": "utf-8", "value": ""})
    );
}
