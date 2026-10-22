//! Tests for `triggers list`, `get`, and `events`.

use pipedream_connect::resources::deployed_triggers::DeployedTriggersApiMock;
use pipedream_connect_types::{
    Data, EmittedEvent, Emitter, EmitterType, ExternalUserId, ListTriggerEventsRequest, Page,
    PageInfo, UnknownFields,
};
use serde_json::json;
use unimock::{MockFn, matching};

use crate::report::exit::Exit;
use crate::test_support::{Env, connected, deployed_triggers_handle, printed, run};

use super::trigger;

const KEY: &str = "MARKER-webhook-signing-key";
const HOOK_KEY: &str = "MARKER-destination-signing-key";

fn deployed() -> Emitter {
    serde_json::from_value(json!({
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
        "webhook_signing_key": KEY,
        "webhooks": [{
            "id": "hook_1",
            "url": "https://example.com/hook",
            "signing_key": HOOK_KEY,
            "signing_key_set": true,
        }],
    }))
    .unwrap()
}

fn assert_redacted(stdout: &str) {
    assert!(
        !stdout.contains(KEY) && !stdout.contains(HOOK_KEY),
        "{stdout}"
    );
    assert_eq!(stdout.matches("\"[REDACTED]\"").count(), 2, "{stdout}");
}

#[tokio::test]
async fn triggers_list_maps_the_filters_and_redacts_signing_keys() {
    let page = Page {
        data: vec![deployed()],
        page_info: PageInfo::default(),
        extra: UnknownFields::new(),
    };
    let clauses = (
        connected(),
        deployed_triggers_handle(),
        DeployedTriggersApiMock::list
            .next_call(
                matching!((request) if request.external_user_id.as_str() == "user_1"
                && request.emitter_type == Some(EmitterType::Http)
                && request.page.limit == 5),
            )
            .returns(Ok(page)),
    );
    let args = [
        "triggers",
        "list",
        "--external-user-id",
        "user_1",
        "--emitter-type",
        "http",
        "--limit",
        "5",
    ];
    let ran = run(&args, clauses).await;
    assert_eq!(ran.exit, Exit::Success);
    assert_redacted(&ran.stdout);
}

#[tokio::test]
async fn triggers_get_redacts_signing_keys_unless_show_secrets() {
    for (show, redacted) in [(false, true), (true, false)] {
        let clauses = (
            connected(),
            deployed_triggers_handle(),
            DeployedTriggersApiMock::get
                .next_call(matching!((id, query) if id.as_str() == "dc_Trigger1"
                    && query.external_user_id.as_str() == "user_1"))
                .returns(Ok(Data {
                    data: deployed(),
                    extra: UnknownFields::new(),
                })),
        );
        let mut args = trigger("get");
        if show {
            args.push("--show-secrets");
        }
        let ran = run(&args, clauses).await;
        assert_eq!(ran.exit, Exit::Success);
        if redacted {
            assert_redacted(&ran.stdout);
        } else {
            assert!(ran.stdout.contains(KEY) && ran.stdout.contains(HOOK_KEY));
        }
    }
}

fn events_request(count: u16) -> ListTriggerEventsRequest {
    ListTriggerEventsRequest::new(ExternalUserId::new("user_1").unwrap(), count).unwrap()
}

#[tokio::test]
async fn triggers_events_sends_the_pipedream_default_or_the_limit() {
    let event: EmittedEvent = serde_json::from_value(json!({
        "e": {"summary": "Standup"}, "k": "emit", "ts": 1_700_000_000_000_i64, "id": "event_1",
    }))
    .unwrap();
    let response = Data {
        data: vec![event],
        extra: UnknownFields::new(),
    };
    let clauses = (
        connected(),
        deployed_triggers_handle(),
        DeployedTriggersApiMock::list_events
            .next_call(matching!((_, request) if *request == events_request(20)))
            .returns(Ok(response.clone())),
    );
    let ran = run(&trigger("events"), clauses).await;
    assert_eq!(ran.stdout, printed(&response));
    let clauses = (
        connected(),
        deployed_triggers_handle(),
        DeployedTriggersApiMock::list_events
            .next_call(matching!((_, request) if *request == events_request(5)))
            .returns(Ok(response)),
    );
    let mut args = trigger("events");
    args.extend(["--limit", "5"]);
    assert_eq!(run(&args, clauses).await.exit, Exit::Success);
}

#[tokio::test]
async fn trigger_commands_reject_invalid_input_before_any_request() {
    let mut over = trigger("events");
    over.extend(["--limit", "101"]);
    let mut negative = trigger("events");
    negative.extend(["--limit", "-3"]);
    let cases = [
        (over, "error: --limit must be from 1 to 100.\n"),
        (negative, "error: --limit must be from 1 to 100.\n"),
        (
            vec![
                "triggers",
                "get",
                "trigger-1",
                "--external-user-id",
                "user_1",
            ],
            "error: <TRIGGER> is not a valid deployed trigger ID. Use an ID that starts with \
             dc_, hi_, or ti_.\n",
        ),
    ];
    for (args, stderr) in cases {
        let ran = run(&args, Env::CONFIGURED.clauses()).await;
        assert_eq!(ran.exit, Exit::Usage, "{args:?}");
        assert_eq!(ran.stderr, stderr, "{args:?}");
    }
}

#[tokio::test]
async fn a_timer_emitter_id_reaches_the_events_request() {
    let clauses = (
        connected(),
        deployed_triggers_handle(),
        DeployedTriggersApiMock::list_events
            .next_call(matching!((id, _) if id.as_str() == "ti_Timer1"))
            .returns(Ok(Data {
                data: Vec::new(),
                extra: UnknownFields::new(),
            })),
    );
    let args = [
        "triggers",
        "events",
        "ti_Timer1",
        "--external-user-id",
        "user_1",
    ];
    assert_eq!(run(&args, clauses).await.exit, Exit::Success);
}
