//! Tests for proxy results and proxy failures.

use http::{HeaderMap, HeaderValue, StatusCode};
use pipedream_connect::error::{Error, InvalidRequest};
use pipedream_connect::resources::proxy::{ProxyApiMock, ProxyResponse};
use serde_json::{Value, json};
use unimock::{Clause, MockFn, matching};

use crate::report::exit::Exit;
use crate::test_support::{connected, proxy_handle, run, run_closed};

use super::proxy;

fn sent(result: pipedream_connect::error::Result<ProxyResponse>) -> impl Clause {
    (
        connected(),
        proxy_handle(),
        ProxyApiMock::send.next_call(matching!(_)).returns(result),
    )
}

#[tokio::test]
async fn an_upstream_error_status_is_a_result_that_exits_zero() {
    let mut headers = HeaderMap::new();
    headers.append("content-type", HeaderValue::from_static("application/json"));
    let response = ProxyResponse {
        status: StatusCode::NOT_FOUND,
        headers,
        body: br#"{"ok":false,"error":"channel_not_found"}"#.to_vec(),
    };
    let ran = run(
        &proxy("get", "/api/conversations.info", &[]),
        sent(Ok(response)),
    )
    .await;
    assert_eq!(ran.exit, Exit::Success);
    assert!(ran.stderr.is_empty());
    let printed: Value = serde_json::from_str(&ran.stdout).unwrap();
    assert_eq!(
        printed,
        json!({
            "status": 404,
            "headers": [{"name": "content-type", "encoding": "utf-8", "value": "application/json"}],
            "body": {"encoding": "utf-8", "value": "{\"ok\":false,\"error\":\"channel_not_found\"}"},
        })
    );
}

#[tokio::test]
async fn a_binary_body_prints_as_base64() {
    let response = ProxyResponse {
        status: StatusCode::OK,
        headers: HeaderMap::new(),
        body: vec![0x89, b'P', b'N', b'G'],
    };
    let ran = run(&proxy("get", "/files/logo.png", &[]), sent(Ok(response))).await;
    let printed: Value = serde_json::from_str(&ran.stdout).unwrap();
    assert_eq!(
        printed["body"],
        json!({"encoding": "base64", "value": "iVBORw=="})
    );
}

#[tokio::test]
async fn an_invalid_target_exits_two() {
    let failed = Err(Error::InvalidRequest(InvalidRequest::ProxyTarget));
    let ran = run(
        &proxy("get", "http://insecure.example.com", &[]),
        sent(failed),
    )
    .await;
    assert_eq!(ran.exit, Exit::Usage);
    assert_eq!(
        ran.stderr,
        "error: <URL> must be an HTTPS URL with a host, or a path that starts with one /. \
         Remove any surrounding spaces, control characters, backslashes, credentials, and \
         fragment.\n"
    );
}

#[tokio::test]
async fn an_unanswered_write_warns_that_the_upstream_request_may_have_run() {
    let ran = run(
        &proxy("post", "/api/chat.postMessage", &[]),
        sent(Err(Error::Unavailable {
            status: Some(504),
            retry_after: None,
        })),
    )
    .await;
    assert_eq!(ran.exit, Exit::Unavailable);
    assert_eq!(
        ran.stderr,
        "error: Pipedream at https://api.pipedream.com is unavailable with status 504. The \
         upstream request may have run. Check its effects before you send it again.\n"
    );
}

#[tokio::test]
async fn an_unanswered_read_suggests_a_retry() {
    let ran = run(
        &proxy("get", "/api/users.list", &[]),
        sent(Err(Error::Unavailable {
            status: None,
            retry_after: None,
        })),
    )
    .await;
    assert_eq!(ran.exit, Exit::Unavailable);
    assert_eq!(
        ran.stderr,
        "error: The CLI could not reach Pipedream at https://api.pipedream.com. Check the \
         network and the API origin, then try again.\n"
    );
}

#[tokio::test]
async fn a_failed_execution_names_the_proxy_request() {
    let error = serde_json::from_value(json!({
        "attribution": {"origin": "upstream_api", "last_call": {"status": 401}},
        "message": "token xoxb-hidden revoked",
    }))
    .unwrap();
    let failed = Err(Error::ExecutionFailed {
        status: 400,
        error: Box::new(error),
    });
    let ran = run(&proxy("delete", "/api/items/1", &[]), sent(failed)).await;
    assert_eq!(ran.exit, Exit::Rejected);
    assert_eq!(
        ran.stderr,
        "error: The proxy request failed at the app API with status 401. Check the account and \
         the request, then run the command again.\n"
    );
}

fn ok() -> ProxyResponse {
    ProxyResponse {
        status: StatusCode::OK,
        headers: HeaderMap::new(),
        body: br#"{"ok":true}"#.to_vec(),
    }
}

#[tokio::test]
async fn a_closed_stdout_after_a_write_says_that_the_upstream_request_ran() {
    let ran = run_closed(&proxy("post", "/api/chat.postMessage", &[]), sent(Ok(ok()))).await;
    assert_eq!(ran.exit, Exit::Internal);
    assert_eq!(
        ran.stderr,
        "error: The CLI cannot write to standard output. The upstream request ran. Check its \
         effects before you send it again.\n"
    );
}

#[tokio::test]
async fn a_closed_stdout_after_a_read_asks_for_an_open_stream() {
    let ran = run_closed(&proxy("get", "/api/users.list", &[]), sent(Ok(ok()))).await;
    assert_eq!(ran.exit, Exit::Internal);
    assert_eq!(
        ran.stderr,
        "error: The CLI cannot write to standard output. Check that the output stream is \
         open.\n"
    );
}
