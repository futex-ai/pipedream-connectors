//! Tests for `triggers deploy`: request mapping, redaction, and validation order.

use std::path::PathBuf;

use pipedream_connect::error::Error;
use pipedream_connect::resources::triggers::TriggersApiMock;
use pipedream_connect_types::{
    ComponentKey, Data, DeployTriggerRequest, DeployTriggerResponse, ExternalUserId, UnknownFields,
    WorkflowEndpoint,
};
use serde_json::json;
use unimock::{Clause, MockFn, matching};

use crate::inputs::{InputSource, ReadInputMock};
use crate::report::exit::Exit;
use crate::test_support::{Env, connected, connects, run, triggers_handle};

const PROPS: &[u8] =
    br#"{"googleCalendar": {"authProvisionId": "apn_Account1"}, "calendarId": "primary"}"#;
const KEY: &str = "MARKER-webhook-signing-key";
const HOOK_KEY: &str = "MARKER-destination-signing-key";
const COMPONENT: &str = "google_calendar-new-or-updated-event-instant";

fn response() -> DeployTriggerResponse {
    Data {
        data: serde_json::from_value(json!({
            "type": "DeployedComponent",
            "id": "dc_Trigger1",
            "owner_id": "exu_1",
            "component_id": "sc_1",
            "configurable_props": [],
            "configured_props": {"calendarId": "primary"},
            "active": true,
            "created_at": 1_700_000_000_000_i64,
            "updated_at": 1_700_000_000_000_i64,
            "name": "New event",
            "name_slug": "new-event",
            "webhook_signing_key": KEY,
            "webhooks": [{"id": "hook_1", "url": "https://example.com/hook",
                          "signing_key": HOOK_KEY, "signing_key_set": true}],
        }))
        .unwrap(),
        extra: UnknownFields::new(),
    }
}

fn full_request() -> DeployTriggerRequest {
    DeployTriggerRequest {
        id: ComponentKey::new(COMPONENT).unwrap(),
        version: Some("0.1.4".to_owned()),
        external_user_id: ExternalUserId::new("user_1").unwrap(),
        configured_props: Some(serde_json::from_slice(PROPS).unwrap()),
        dynamic_props_id: Some("dyp_1".to_owned()),
        workflow_id: Some(WorkflowEndpoint::new("p_abc123").unwrap()),
        webhook_url: Some("https://example.com/hook".to_owned()),
        emit_on_deploy: Some(false),
    }
}

fn deploy<'a>(extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec![
        "triggers",
        "deploy",
        COMPONENT,
        "--external-user-id",
        "user_1",
    ];
    args.extend_from_slice(extra);
    args
}

fn deployed(result: pipedream_connect::error::Result<DeployTriggerResponse>) -> impl Clause {
    (
        connected(),
        triggers_handle(),
        TriggersApiMock::deploy
            .next_call(matching!(_))
            .returns(result),
    )
}

#[tokio::test]
async fn triggers_deploy_maps_every_flag_and_redacts_signing_keys() {
    let clauses = (
        Env::CONFIGURED.clauses(),
        ReadInputMock
            .next_call(
                matching!((source) if **source == InputSource::File(PathBuf::from("props.json"))),
            )
            .returns(Ok(PROPS.to_vec())),
        connects(),
        triggers_handle(),
        TriggersApiMock::deploy
            .next_call(matching!(eq!(&full_request())))
            .returns(Ok(response())),
    );
    let args = deploy(&[
        "--props",
        "props.json",
        "--version",
        "0.1.4",
        "--dynamic-props-id",
        "dyp_1",
        "--workflow-id",
        "p_abc123",
        "--webhook-url",
        "https://example.com/hook",
        "--emit-on-deploy",
        "false",
    ]);
    let ran = run(&args, clauses).await;
    assert_eq!(ran.exit, Exit::Success, "{}", ran.stderr);
    assert!(!ran.stdout.contains(KEY) && !ran.stdout.contains(HOOK_KEY));
    assert_eq!(ran.stdout.matches("\"[REDACTED]\"").count(), 2);
}

#[tokio::test]
async fn show_secrets_prints_the_new_signing_keys() {
    let ran = run(&deploy(&["--show-secrets"]), deployed(Ok(response()))).await;
    assert_eq!(ran.exit, Exit::Success);
    assert!(ran.stdout.contains(KEY) && ran.stdout.contains(HOOK_KEY));
}

#[tokio::test]
async fn a_loopback_http_webhook_is_accepted() {
    let clauses = (
        connected(),
        triggers_handle(),
        TriggersApiMock::deploy
            .next_call(matching!((request) if request.webhook_url.as_deref()
                == Some("http://localhost:8080/hook")))
            .returns(Ok(response())),
    );
    let ran = run(
        &deploy(&["--webhook-url", "http://localhost:8080/hook"]),
        clauses,
    )
    .await;
    assert_eq!(ran.exit, Exit::Success);
}

#[tokio::test]
async fn invalid_deployments_fail_before_reading_props_or_any_request() {
    let cases = [
        (
            deploy(&[
                "--webhook-url",
                "ftp://example.com/hook",
                "--props",
                "props.json",
            ]),
            "error: --webhook-url must be an HTTPS URL or an HTTP URL on localhost.\n",
        ),
        (
            deploy(&["--workflow-id", "p/abc", "--props", "props.json"]),
            "error: --workflow-id is not a valid workflow ID. Use 1 to 250 visible ASCII \
             characters without /, \\, ?, or #.\n",
        ),
    ];
    for (args, stderr) in cases {
        let ran = run(&args, Env::CONFIGURED.clauses()).await;
        assert_eq!(ran.exit, Exit::Usage, "{args:?}");
        assert_eq!(ran.stderr, stderr, "{args:?}");
    }
}

#[tokio::test]
async fn an_unanswered_deployment_warns_that_the_trigger_may_exist() {
    let ran = run(
        &deploy(&[]),
        deployed(Err(Error::Unavailable { status: Some(502) })),
    )
    .await;
    assert_eq!(ran.exit, Exit::Unavailable);
    assert_eq!(
        ran.stderr,
        "error: Pipedream at https://api.pipedream.com is unavailable with status 502. Pipedream \
         may have deployed the trigger. Check triggers list before you deploy it again.\n"
    );
}
