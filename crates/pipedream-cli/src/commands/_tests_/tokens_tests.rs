//! Tests for `tokens create` against a mocked token trait object.

use pipedream_connect::error::Error;
use pipedream_connect::resources::tokens::TokensApiMock;
use pipedream_connect_types::{ConnectTokenResponse, CreateConnectTokenRequest, ExternalUserId};
use serde_json::json;
use unimock::{Clause, MockFn, matching};

use crate::report::exit::Exit;
use crate::test_support::{Env, connected, printed, run, run_closed, tokens_handle};

const TOKEN: &str = "ctok_0123456789abcdef0123456789abcdef";

const LINK: &str =
    "https://pipedream.com/_static/connect.html?token=ctok_0123456789abcdef0123456789abcdef";

fn response() -> ConnectTokenResponse {
    serde_json::from_value(json!({
        "token": TOKEN,
        "connect_link_url": LINK,
        "expires_at": "2026-10-21T04:00:00Z",
    }))
    .unwrap()
}

fn user() -> ExternalUserId {
    ExternalUserId::new("user_1").unwrap()
}

fn full_request() -> CreateConnectTokenRequest {
    let mut request = CreateConnectTokenRequest::new(user())
        .with_expires_in(600)
        .unwrap();
    request.allowed_origins = vec![
        "https://app.example.com".to_owned(),
        "https://admin.example.com".to_owned(),
    ];
    request.success_redirect_uri = Some("https://app.example.com/done".to_owned());
    request.error_redirect_uri = Some("https://app.example.com/failed".to_owned());
    request.webhook_uri = Some("https://hooks.example.com/connect".to_owned());
    request.scope = Some("connect:*".to_owned());
    request.allow_progressive_scopes = Some(false);
    request
}

fn created(result: pipedream_connect::error::Result<ConnectTokenResponse>) -> impl Clause {
    (
        connected(),
        tokens_handle(),
        TokensApiMock::create
            .next_call(matching!(eq!(&CreateConnectTokenRequest::new(user()))))
            .returns(result),
    )
}

#[tokio::test]
async fn tokens_create_maps_every_flag_and_prints_the_token_and_the_link() {
    let clauses = (
        connected(),
        tokens_handle(),
        TokensApiMock::create
            .next_call(matching!(eq!(&full_request())))
            .returns(Ok(response())),
    );
    let args = [
        "tokens",
        "create",
        "--external-user-id",
        "user_1",
        "--expires-in",
        "600",
        "--allowed-origin",
        "https://app.example.com",
        "--allowed-origin",
        "https://admin.example.com",
        "--success-redirect-uri",
        "https://app.example.com/done",
        "--error-redirect-uri",
        "https://app.example.com/failed",
        "--webhook-uri",
        "https://hooks.example.com/connect",
        "--scope",
        "connect:*",
        "--allow-progressive-scopes",
        "false",
    ];
    let ran = run(&args, clauses).await;
    assert_eq!(ran.exit, Exit::Success);
    assert_eq!(ran.stdout, printed(&response()));
    for value in [TOKEN, LINK, "2026-10-21T04:00:00Z"] {
        assert!(ran.stdout.contains(value), "{value}");
    }
    assert!(!ran.stdout.contains("[REDACTED]"));
}

#[tokio::test]
async fn tokens_create_without_options_uses_the_sdk_defaults() {
    let ran = run(
        &["tokens", "create", "--external-user-id", "user_1"],
        created(Ok(response())),
    )
    .await;
    assert_eq!(ran.exit, Exit::Success);
    assert!(ran.stdout.contains(TOKEN));
}

#[tokio::test]
async fn tokens_create_rejects_invalid_input_before_any_request() {
    let cases = [
        (
            &["--expires-in", "0"][..],
            "error: --expires-in must be from 1 to 14400.\n",
        ),
        (
            &["--expires-in", "14401"],
            "error: --expires-in must be from 1 to 14400.\n",
        ),
        (
            &["--expires-in", "soon"],
            "error: --expires-in must be from 1 to 14400.\n",
        ),
        (
            &["--success-redirect-uri", "http://app.example.com/done"],
            "error: --success-redirect-uri must be an HTTPS URL with a host.\n",
        ),
        (
            &["--error-redirect-uri", "ftp://app.example.com/failed"],
            "error: --error-redirect-uri must be an HTTPS URL with a host.\n",
        ),
        (
            &["--webhook-uri", "not a url"],
            "error: --webhook-uri must be an HTTPS URL with a host.\n",
        ),
    ];
    for (extra, stderr) in cases {
        let mut args = vec!["tokens", "create", "--external-user-id", "user_1"];
        args.extend_from_slice(extra);
        let ran = run(&args, Env::CONFIGURED.clauses()).await;
        assert_eq!(ran.exit, Exit::Usage, "{extra:?}");
        assert_eq!(ran.stderr, stderr, "{extra:?}");
    }
}

#[tokio::test]
async fn an_unanswered_token_request_warns_that_a_token_may_exist() {
    let ran = run(
        &["tokens", "create", "--external-user-id", "user_1"],
        created(Err(Error::Unavailable {
            status: None,
            retry_after: None,
        })),
    )
    .await;
    assert_eq!(ran.exit, Exit::Unavailable);
    assert_eq!(
        ran.stderr,
        "error: The CLI could not reach Pipedream at https://api.pipedream.com. Pipedream may \
         have created the token. Run the command again for a new one.\n"
    );
}

#[tokio::test]
async fn a_closed_stdout_after_a_token_says_that_pipedream_created_it() {
    let ran = run_closed(
        &["tokens", "create", "--external-user-id", "user_1"],
        created(Ok(response())),
    )
    .await;
    assert_eq!(ran.exit, Exit::Internal);
    assert_eq!(
        ran.stderr,
        "error: The CLI cannot write to standard output. Pipedream created the token. Run \
         the command again for a new one.\n"
    );
}
