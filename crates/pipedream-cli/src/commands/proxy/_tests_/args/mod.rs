//! Tests for the proxy commands against a mocked proxy trait object.

mod request_tests;
mod response_tests;

use pipedream_connect_types::{AccountId, AccountRef, ExternalUserId};

/// The account that every proxy test uses.
fn account() -> AccountRef {
    AccountRef::new(
        ExternalUserId::new("user_1").unwrap(),
        AccountId::new("apn_Account1").unwrap(),
    )
}

/// Builds a proxy command line for `method` and `url` with the test account.
fn proxy<'a>(method: &'a str, url: &'a str, extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec![
        "proxy",
        method,
        url,
        "--external-user-id",
        "user_1",
        "--account",
        "apn_Account1",
    ];
    args.extend_from_slice(extra);
    args
}
