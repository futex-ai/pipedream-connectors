//! Tests for `accounts list` and `accounts get`, including credential redaction.

use pipedream_connect::error::Error;
use pipedream_connect::resources::accounts::AccountsApiMock;
use pipedream_connect_types::{
    Account, AccountId, AccountRef, AppSlug, ExternalUserId, ListAccountsRequest, OauthAppId, Page,
    PageInfo, PageRequest, UnknownFields,
};
use serde_json::json;
use unimock::{MockFn, matching};

use crate::report::exit::Exit;
use crate::test_support::{Env, accounts_handle, connected, run};

const ACCESS: &str = "MARKER-oauth-access-token";
const REFRESH: &str = "MARKER-oauth-refresh-token";

fn account() -> Account {
    serde_json::from_value(json!({
        "id": "apn_Account1",
        "external_id": "user_1",
        "credentials": {"oauth_access_token": ACCESS, "oauth_refresh_token": REFRESH},
    }))
    .unwrap()
}

fn page() -> Page<Account> {
    Page {
        data: vec![account()],
        page_info: PageInfo {
            count: Some(1),
            ..PageInfo::default()
        },
        extra: UnknownFields::new(),
    }
}

fn filtered_request() -> ListAccountsRequest {
    ListAccountsRequest {
        page: PageRequest::with_limit(10).unwrap(),
        external_user_id: Some(ExternalUserId::new("user_1").unwrap()),
        app: Some(AppSlug::new("google_calendar").unwrap()),
        oauth_app_id: Some(OauthAppId::new("oa_Custom1").unwrap()),
        include_credentials: Some(true),
    }
}

fn owned() -> AccountRef {
    AccountRef::new(
        ExternalUserId::new("user_1").unwrap(),
        AccountId::new("apn_Account1").unwrap(),
    )
}

fn assert_redacted(stdout: &str) {
    assert!(
        stdout.contains("\"credentials\": \"[REDACTED]\""),
        "{stdout}"
    );
    assert!(!stdout.contains(ACCESS) && !stdout.contains(REFRESH));
}

#[tokio::test]
async fn accounts_list_maps_every_filter_and_redacts_credentials() {
    let clauses = (
        connected(),
        accounts_handle(),
        AccountsApiMock::list
            .next_call(matching!(eq!(&filtered_request())))
            .returns(Ok(page())),
    );
    let args = [
        "accounts",
        "list",
        "--external-user-id",
        "user_1",
        "--app",
        "google_calendar",
        "--oauth-app-id",
        "oa_Custom1",
        "--include-credentials",
        "--limit",
        "10",
    ];
    let ran = run(&args, clauses).await;
    assert_eq!(ran.exit, Exit::Success);
    assert_redacted(&ran.stdout);
}

#[tokio::test]
async fn show_secrets_prints_account_credentials() {
    let clauses = (
        connected(),
        accounts_handle(),
        AccountsApiMock::list
            .next_call(matching!((request) if request.include_credentials.is_none()))
            .returns(Ok(page())),
    );
    let ran = run(&["accounts", "list", "--show-secrets"], clauses).await;
    assert_eq!(ran.exit, Exit::Success);
    assert!(ran.stdout.contains(ACCESS) && ran.stdout.contains(REFRESH));
    assert!(!ran.stdout.contains("[REDACTED]"));
}

#[tokio::test]
async fn accounts_get_checks_ownership_when_given_an_external_user() {
    let clauses = (
        connected(),
        accounts_handle(),
        AccountsApiMock::get_owned
            .next_call(matching!((account, request) if *account == owned()
                && request.include_credentials == Some(true)))
            .returns(Ok(account())),
    );
    let args = [
        "accounts",
        "get",
        "apn_Account1",
        "--external-user-id",
        "user_1",
        "--include-credentials",
    ];
    let ran = run(&args, clauses).await;
    assert_eq!(ran.exit, Exit::Success);
    assert_redacted(&ran.stdout);
}

#[tokio::test]
async fn accounts_get_reads_by_id_without_an_external_user() {
    let clauses = (
        connected(),
        accounts_handle(),
        AccountsApiMock::get
            .next_call(matching!((id, request) if id.as_str() == "apn_Account1"
                && request.include_credentials.is_none()))
            .returns(Err(Error::NotFound { message: None })),
    );
    let ran = run(&["accounts", "get", "apn_Account1"], clauses).await;
    assert_eq!(ran.exit, Exit::NotFound);
    assert_eq!(
        ran.stderr,
        "error: Pipedream did not find the account. Check the account ID and the external user \
         ID.\n"
    );
}

#[tokio::test]
async fn accounts_commands_reject_invalid_input_before_any_request() {
    let cases = [
        (
            &["accounts", "get", "account-1"][..],
            "error: <ACCOUNT> is not a valid account ID. Use an ID that starts with apn_.\n",
        ),
        (
            &["accounts", "list", "--oauth-app-id", "custom"],
            "error: --oauth-app-id is not a valid OAuth app ID. Use an ID that starts with oa_.\n",
        ),
        (
            &["accounts", "list", "--external-user-id", "user/1"],
            "error: --external-user-id is not a valid external user ID. Use 1 to 250 visible \
             ASCII characters without /, \\, ?, or #.\n",
        ),
    ];
    for (args, stderr) in cases {
        let ran = run(args, Env::CONFIGURED.clauses()).await;
        assert_eq!(ran.exit, Exit::Usage, "{args:?}");
        assert_eq!(ran.stderr, stderr, "{args:?}");
    }
}
