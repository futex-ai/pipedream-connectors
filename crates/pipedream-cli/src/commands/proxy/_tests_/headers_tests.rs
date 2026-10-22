//! Tests for `--header` parsing and its agreement with the SDK header policy.

use std::sync::Arc;

use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use pipedream_connect::client::RequestDispatcherMock;
use pipedream_connect::error::InvalidRequest;
use pipedream_connect::resources::proxy::{Proxy, ProxyApi, ProxyRequest};
use pipedream_connect::transport::HttpResponse;
use pipedream_connect_types::{AccountId, AccountRef, ExternalUserId};
use unimock::{MockFn, Unimock, matching};

use crate::commands::proxy::headers::{RESERVED, RESERVED_PREFIXES, parse};
use crate::error::{Error, HeaderProblem};

fn arguments(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn problem(values: &[&str]) -> (usize, HeaderProblem) {
    match parse(&arguments(values)).unwrap_err() {
        Error::InvalidHeader { position, problem } => (position, problem),
        error => panic!("expected a header error, got {error:?}"),
    }
}

/// Every reserved name, plus one sample name for each reserved prefix.
fn reserved_samples() -> Vec<String> {
    RESERVED
        .iter()
        .map(|name| (*name).to_owned())
        .chain(
            RESERVED_PREFIXES
                .iter()
                .map(|prefix| format!("{prefix}sample")),
        )
        .collect()
}

fn sdk_request(name: &str) -> ProxyRequest {
    let account = AccountRef::new(
        ExternalUserId::new("user_1").unwrap(),
        AccountId::new("apn_Account1").unwrap(),
    );
    ProxyRequest::get(account, "https://example.com/api").with_header(
        HeaderName::from_bytes(name.as_bytes()).unwrap(),
        HeaderValue::from_static("1"),
    )
}

#[test]
fn headers_keep_repeated_names_and_trim_spaces() {
    let headers = parse(&arguments(&[
        "Content-Type: application/json",
        "X-Trace:  one ",
        "x-trace: two",
    ]))
    .unwrap();
    assert_eq!(headers["content-type"], "application/json");
    let traces: Vec<_> = headers.get_all("x-trace").iter().collect();
    assert_eq!(traces, ["one", "two"]);
}

#[test]
fn malformed_headers_are_named_by_position_without_their_value() {
    assert_eq!(
        problem(&["X-Ok: 1", "Authorization Bearer sk_hidden"]),
        (2, HeaderProblem::Format)
    );
    assert_eq!(problem(&[": no name"]), (1, HeaderProblem::Format));
    assert_eq!(problem(&["Bad Name: x"]), (1, HeaderProblem::Name));
    assert_eq!(
        problem(&["X-Ok: a\u{1}b"]),
        (
            1,
            HeaderProblem::Value {
                name: "x-ok".to_owned()
            }
        )
    );
}

#[test]
fn every_reserved_name_and_prefix_is_named_in_lowercase() {
    for name in reserved_samples() {
        let upper = format!("{}: session=sk_hidden", name.to_uppercase());
        assert_eq!(
            problem(&[upper.as_str()]),
            (1, HeaderProblem::Reserved { name: name.clone() }),
            "{name}"
        );
    }
}

#[tokio::test]
async fn the_sdk_rejects_every_header_that_the_cli_reserves() {
    for name in reserved_samples() {
        let proxy = Proxy::new(Arc::new(Unimock::new(())));
        let result = proxy.send(sdk_request(&name)).await;
        assert!(
            matches!(
                result,
                Err(pipedream_connect::error::Error::InvalidRequest(
                    InvalidRequest::ProxyHeader
                ))
            ),
            "{name}: {result:?}"
        );
    }
}

#[tokio::test]
async fn the_sdk_forwards_a_header_that_the_cli_accepts() {
    assert!(parse(&arguments(&["X-Api-Key: 1"])).is_ok());
    let dispatcher = Unimock::new(
        RequestDispatcherMock::dispatch
            .next_call(matching!(_))
            .returns(Ok(HttpResponse {
                status: StatusCode::OK,
                headers: HeaderMap::new(),
                body: Vec::new(),
            })),
    );
    let proxy = Proxy::new(Arc::new(dispatcher));
    assert!(proxy.send(sdk_request("x-api-key")).await.is_ok());
}
