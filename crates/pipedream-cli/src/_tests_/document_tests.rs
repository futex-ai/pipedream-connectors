//! Tests for document encoding and the secret output policy.

use pipedream_connect_types::Account;
use serde_json::json;

use crate::document::{Document, Output, Secrets};
use crate::redact::REDACTED;

fn account() -> Account {
    serde_json::from_value(json!({
        "id": "apn_Account1",
        "credentials": {"oauth_access_token": "token-value"},
    }))
    .unwrap()
}

fn text(document: &Document) -> String {
    String::from_utf8(document.bytes().to_vec()).unwrap()
}

#[test]
fn redact_policy_replaces_credentials_in_the_document() {
    let document = Document::redacted(account(), Secrets::Redact).unwrap();
    let text = text(&document);
    assert!(text.contains(REDACTED));
    assert!(!text.contains("token-value"));
}

#[test]
fn show_policy_keeps_credentials_in_the_document() {
    let document = Document::redacted(account(), Secrets::Show).unwrap();
    let text = text(&document);
    assert!(text.contains("token-value"));
    assert!(!text.contains(REDACTED));
}

#[test]
fn revealed_documents_are_pretty_json_without_a_trailing_newline() {
    let document = Document::revealed(&account()).unwrap();
    assert_eq!(
        document.bytes(),
        serde_json::to_vec_pretty(&account()).unwrap()
    );
    assert!(!text(&document).ends_with('\n'));
}

#[test]
fn a_document_becomes_an_output_without_notices() {
    let output = Output::from(Document::revealed(&json!({"ok": true})).unwrap());
    assert!(output.notices.is_empty());
}
