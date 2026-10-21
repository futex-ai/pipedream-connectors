//! Marker values in every credential-class field of every model that a command prints.
//!
//! Commands print these models with credential-class fields: accounts, deployed
//! trigger emitters, and Connect token responses. Each may appear alone, in a
//! page, in an envelope, or in an `--all` traversal.

use pipedream_connect_types::{Account, ConnectTokenResponse, Data, Emitter, Page};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::commands::pages::{End, Traversal};
use crate::document::{Document, Secrets};
use crate::redact::Redact;

const CREDENTIAL: &str = "MARKER-account-credential";
const SIGNING_KEY: &str = "MARKER-webhook-signing-key";
const DESTINATION_KEY: &str = "MARKER-destination-signing-key";
const TOKEN: &str = "ctok_0123456789abcdef0123456789abcdef";
const LINK: &str = "https://pipedream.com/_static/connect.html?token=MARKER-link";

fn account() -> Value {
    json!({
        "id": "apn_Account1",
        "credentials": {"oauth_access_token": CREDENTIAL, "nested": {"key": CREDENTIAL}},
    })
}

fn emitter() -> Value {
    json!({
        "type": "DeployedComponent",
        "id": "dc_Trigger1",
        "owner_id": "exu_1",
        "component_id": "sc_1",
        "configurable_props": [],
        "configured_props": {},
        "active": true,
        "created_at": 1_700_000_000_000_i64,
        "updated_at": 1_700_000_000_000_i64,
        "name": "New event",
        "name_slug": "new-event",
        "webhook_signing_key": SIGNING_KEY,
        "webhooks": [
            {"id": "hook_1", "url": "https://example.com/1", "signing_key": DESTINATION_KEY,
             "signing_key_set": true},
            {"id": "hook_2", "url": "https://example.com/2", "signing_key": DESTINATION_KEY,
             "signing_key_set": true},
        ],
    })
}

fn text<T: Serialize + Redact>(model: T, secrets: Secrets) -> String {
    String::from_utf8(Document::redacted(model, secrets).unwrap().bytes().to_vec()).unwrap()
}

fn check<T: DeserializeOwned + Serialize + Redact>(raw: Value, markers: &[&str]) {
    let redacted = text(
        serde_json::from_value::<T>(raw.clone()).unwrap(),
        Secrets::Redact,
    );
    let shown = text(serde_json::from_value::<T>(raw).unwrap(), Secrets::Show);
    for marker in markers {
        assert!(!redacted.contains(marker), "{marker} printed in {redacted}");
        assert!(shown.contains(marker), "{marker} missing from {shown}");
    }
}

fn traversal<T: DeserializeOwned>(record: Value) -> Traversal<T> {
    Traversal {
        data: vec![serde_json::from_value(record).unwrap()],
        pages: 1,
        end: End::Exhausted,
        next: None,
    }
}

#[test]
fn account_credentials_are_redacted_in_every_printed_form() {
    check::<Account>(account(), &[CREDENTIAL]);
    check::<Page<Account>>(
        json!({"data": [account(), account()], "page_info": {}}),
        &[CREDENTIAL],
    );
    let all: Traversal<Account> = traversal(account());
    assert!(!text(all, Secrets::Redact).contains(CREDENTIAL));
}

#[test]
fn trigger_signing_keys_are_redacted_in_every_printed_form() {
    let markers = [SIGNING_KEY, DESTINATION_KEY];
    check::<Data<Emitter>>(json!({"data": emitter()}), &markers);
    check::<Page<Emitter>>(json!({"data": [emitter()], "page_info": {}}), &markers);
    let all: Traversal<Emitter> = traversal(emitter());
    let printed = text(all, Secrets::Redact);
    assert!(!printed.contains(SIGNING_KEY) && !printed.contains(DESTINATION_KEY));
}

#[test]
fn the_token_command_reveals_the_token_and_the_link() {
    let response: ConnectTokenResponse = serde_json::from_value(json!({
        "token": TOKEN,
        "connect_link_url": LINK,
        "expires_at": "2026-10-21T04:00:00Z",
    }))
    .unwrap();
    let printed =
        String::from_utf8(Document::revealed(&response).unwrap().bytes().to_vec()).unwrap();
    assert!(printed.contains(TOKEN) && printed.contains(LINK));
}
