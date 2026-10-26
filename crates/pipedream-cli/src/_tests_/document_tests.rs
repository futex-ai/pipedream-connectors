//! Tests for result encoding, the secret output policy, and the result target.

use std::collections::BTreeMap;

use pipedream_connect_types::Account;
use serde_json::json;

use crate::document::{Output, Secrets};
use crate::error::Error;
use crate::redact::REDACTED;
use crate::target::Target;

fn account() -> Account {
    serde_json::from_value(json!({
        "id": "apn_Account1",
        "credentials": {"oauth_access_token": "token-value"},
    }))
    .unwrap()
}

fn text(output: &Output) -> String {
    String::from_utf8(output.document.bytes().to_vec()).unwrap()
}

#[test]
fn redact_policy_replaces_credentials_in_the_document() {
    let output = Output::redacted(Target::Account, account(), Secrets::Redact).unwrap();
    let text = text(&output);
    assert!(text.contains(REDACTED));
    assert!(!text.contains("token-value"));
}

#[test]
fn show_policy_keeps_credentials_in_the_document() {
    let output = Output::redacted(Target::Account, account(), Secrets::Show).unwrap();
    let text = text(&output);
    assert!(text.contains("token-value"));
    assert!(!text.contains(REDACTED));
}

#[test]
fn revealed_documents_are_pretty_json_without_a_trailing_newline() {
    let output = Output::revealed(Target::Account, &account()).unwrap();
    assert_eq!(
        output.document.bytes(),
        serde_json::to_vec_pretty(&account()).unwrap()
    );
    assert!(!text(&output).ends_with('\n'));
}

#[test]
fn an_output_names_its_target_and_starts_without_notices() {
    let output = Output::revealed(Target::Apps, &json!({"ok": true})).unwrap();
    assert_eq!(output.target, Target::Apps);
    assert!(output.notices.is_empty());
}

#[test]
fn a_result_that_does_not_encode_names_its_target() {
    let keys = BTreeMap::from([(vec![1_u8], 1_u8)]);
    let error = Output::revealed(Target::TokenCreate, &keys).unwrap_err();
    assert!(matches!(
        error,
        Error::Unencodable {
            target: Target::TokenCreate,
            ..
        }
    ));
}
