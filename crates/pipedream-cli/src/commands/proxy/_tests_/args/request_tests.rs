//! Tests that proxy arguments map to the request and fail before any request.

use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use http::{HeaderMap, StatusCode};
use pipedream_connect::resources::proxy::{ProxyApiMock, ProxyMethod, ProxyRequest, ProxyResponse};
use unimock::{MockFn, matching};

use crate::inputs::{InputSource, ReadInputMock};
use crate::report::exit::Exit;
use crate::test_support::{Env, connected, connects, proxy_handle, run};

use super::{account, proxy};

const BODY: &[u8] = br#"{"channel":"C1","text":"hello"}"#;

fn ok() -> pipedream_connect::error::Result<ProxyResponse> {
    Ok(ProxyResponse {
        status: StatusCode::OK,
        headers: HeaderMap::new(),
        body: Vec::new(),
    })
}

fn is_post_message(request: &ProxyRequest) -> bool {
    request.method == ProxyMethod::Post
        && request.target == "https://slack.com/api/chat.postMessage"
        && request.account == account()
        && request.body.as_deref() == Some(BODY)
        && request.headers["content-type"] == "application/json"
        && request.headers.get_all("x-trace").iter().count() == 2
}

#[tokio::test]
async fn proxy_post_maps_the_target_headers_and_body_file() {
    let clauses = (
        Env::CONFIGURED.clauses(),
        ReadInputMock
            .next_call(
                matching!((source) if **source == InputSource::File(PathBuf::from("body.json"))),
            )
            .returns(Ok(BODY.to_vec())),
        connects(),
        proxy_handle(),
        ProxyApiMock::send
            .next_call(matching!((request) if is_post_message(request)))
            .returns(ok()),
    );
    let args = proxy(
        "post",
        "https://slack.com/api/chat.postMessage",
        &[
            "--header",
            "Content-Type: application/json",
            "--header",
            "X-Trace: one",
            "--header",
            "X-Trace: two",
            "--body",
            "body.json",
        ],
    );
    let ran = run(&args, clauses).await;
    assert_eq!(ran.exit, Exit::Success, "{}", ran.stderr);
}

#[tokio::test]
async fn every_method_maps_to_its_proxy_method_and_reads_a_body_from_stdin() {
    let methods = [
        ("get", ProxyMethod::Get),
        ("put", ProxyMethod::Put),
        ("patch", ProxyMethod::Patch),
        ("delete", ProxyMethod::Delete),
    ];
    for (name, method) in methods {
        let clauses = (
            Env::CONFIGURED.clauses(),
            ReadInputMock
                .next_call(matching!((source) if **source == InputSource::Stdin))
                .returns(Ok(b"raw".to_vec())),
            connects(),
            proxy_handle(),
            ProxyApiMock::send
                .next_call(matching!((request) if request.target == "/v1/items"
                    && request.body.as_deref() == Some(&b"raw"[..])
                    && request.headers.is_empty()))
                .answers_arc(Arc::new(move |_, request| {
                    assert_eq!(request.method, method);
                    ok()
                })),
        );
        let ran = run(&proxy(name, "/v1/items", &["--body", "-"]), clauses).await;
        assert_eq!(ran.exit, Exit::Success, "{name}");
    }
}

#[tokio::test]
async fn proxy_get_without_a_body_sends_none() {
    let clauses = (
        connected(),
        proxy_handle(),
        ProxyApiMock::send
            .next_call(matching!((request) if request.method == ProxyMethod::Get
                && request.body.is_none()))
            .returns(ok()),
    );
    let ran = run(&proxy("get", "https://example.com/api", &[]), clauses).await;
    assert_eq!(ran.exit, Exit::Success);
}

#[tokio::test]
async fn invalid_proxy_input_fails_before_any_request_without_header_values() {
    let cases = [
        (
            proxy("get", "/v1", &["--header", "Cookie: session=sk_hidden"]),
            "error: Pipedream reserves the cookie header, so the proxy does not forward it. \
             Remove it.\n",
        ),
        (
            proxy(
                "get",
                "/v1",
                &["--header", "Authorization Bearer sk_hidden"],
            ),
            "error: --header number 1 must look like 'Name: value'.\n",
        ),
        (
            proxy(
                "get",
                "/v1",
                &["--header", "X-Ok: 1", "--header", "Bad Name: sk_hidden"],
            ),
            "error: --header number 2 has a name that HTTP does not allow. Use letters, digits, \
             and hyphens.\n",
        ),
        (
            proxy("get", "/v1", &["--header", "X-Ok: sk_hidden\u{1}"]),
            "error: The x-ok header has a value that HTTP does not allow. Remove its control \
             characters.\n",
        ),
        (
            vec![
                "proxy",
                "get",
                "/v1",
                "--external-user-id",
                "user_1",
                "--account",
                "acct_1",
            ],
            "error: --account is not a valid account ID. Use an ID that starts with apn_ and \
             then has 1 to 246 ASCII letters or digits.\n",
        ),
    ];
    for (args, stderr) in cases {
        let ran = run(&args, Env::CONFIGURED.clauses()).await;
        assert_eq!(ran.exit, Exit::Usage, "{args:?}");
        assert_eq!(ran.stderr, stderr, "{args:?}");
        assert!(!ran.stderr.contains("sk_hidden"));
    }
}

#[tokio::test]
async fn an_unreadable_body_fails_before_any_request() {
    let clauses = (
        Env::CONFIGURED.clauses(),
        ReadInputMock
            .next_call(matching!(_))
            .answers(&|_, _| Err(io::Error::from(io::ErrorKind::NotFound))),
    );
    let ran = run(&proxy("post", "/v1", &["--body", "missing.bin"]), clauses).await;
    assert_eq!(ran.exit, Exit::Usage);
    assert_eq!(
        ran.stderr,
        "error: The --body file does not exist. Check the path.\n"
    );
}
