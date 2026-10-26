//! Tests for `actions run`: props input, request mapping, results, and failures.

use std::io;
use std::path::PathBuf;

use pipedream_connect::error::Error;
use pipedream_connect::resources::actions::ActionsApiMock;
use pipedream_connect_types::{
    ComponentKey, ConfiguredProps, ExternalUserId, RunActionRequest, RunActionResponse,
    StashReference,
};
use serde_json::json;
use unimock::{Clause, MockFn, matching};

use crate::commands::actions::stash;
use crate::inputs::{InputSource, ReadInputMock};
use crate::report::exit::Exit;
use crate::test_support::{Env, actions_handle, connected, connects, printed, run, run_closed};

const PROPS: &[u8] =
    br#"{"googleCalendar": {"authProvisionId": "apn_Account1"}, "calendarId": "primary"}"#;

fn request(props: bool, stash_id: Option<StashReference>) -> RunActionRequest {
    let configured_props: Option<ConfiguredProps> =
        props.then(|| serde_json::from_slice(PROPS).unwrap());
    RunActionRequest {
        id: ComponentKey::new("google_calendar-list-events").unwrap(),
        version: None,
        external_user_id: ExternalUserId::new("user_1").unwrap(),
        configured_props,
        dynamic_props_id: None,
        stash_id,
    }
}

fn file_request() -> RunActionRequest {
    RunActionRequest {
        version: Some("0.1.2".to_owned()),
        dynamic_props_id: Some("dyp_1".to_owned()),
        ..request(true, Some(StashReference::New))
    }
}

fn response() -> RunActionResponse {
    serde_json::from_value(json!({
        "exports": {"$summary": "Found 2 events"},
        "ret": [{"id": "event_1"}, {"id": "event_2"}],
        "stash_id": null,
    }))
    .unwrap()
}

fn run_args<'a>(extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec![
        "actions",
        "run",
        "google_calendar-list-events",
        "--external-user-id",
        "user_1",
    ];
    args.extend_from_slice(extra);
    args
}

fn ran_with(result: pipedream_connect::error::Result<RunActionResponse>) -> impl Clause {
    (
        connected(),
        actions_handle(),
        ActionsApiMock::run
            .next_call(matching!(eq!(&request(false, None))))
            .returns(result),
    )
}

#[tokio::test]
async fn actions_run_sends_props_from_a_file_and_prints_the_result() {
    let clauses = (
        Env::CONFIGURED.clauses(),
        ReadInputMock
            .next_call(
                matching!((source) if **source == InputSource::File(PathBuf::from("props.json"))),
            )
            .returns(Ok(PROPS.to_vec())),
        connects(),
        actions_handle(),
        ActionsApiMock::run
            .next_call(matching!(eq!(&file_request())))
            .returns(Ok(response())),
    );
    let args = run_args(&[
        "--props",
        "props.json",
        "--version",
        "0.1.2",
        "--dynamic-props-id",
        "dyp_1",
        "--stash-id",
        "NEW",
    ]);
    let ran = run(&args, clauses).await;
    assert_eq!(ran.exit, Exit::Success);
    assert_eq!(ran.stdout, printed(&response()));
    assert!(ran.stderr.is_empty());
}

#[tokio::test]
async fn actions_run_reads_props_from_standard_input() {
    let clauses = (
        Env::CONFIGURED.clauses(),
        ReadInputMock
            .next_call(matching!((source) if **source == InputSource::Stdin))
            .returns(Ok(PROPS.to_vec())),
        connects(),
        actions_handle(),
        ActionsApiMock::run
            .next_call(matching!(eq!(&request(true, None))))
            .returns(Ok(response())),
    );
    let ran = run(&run_args(&["--props", "-"]), clauses).await;
    assert_eq!(ran.exit, Exit::Success);
}

#[test]
fn stash_selectors_map_new_and_booleans_and_keep_other_ids() {
    assert_eq!(stash("NEW".to_owned()), StashReference::New);
    assert_eq!(stash("true".to_owned()), StashReference::Boolean(true));
    assert_eq!(stash("false".to_owned()), StashReference::Boolean(false));
    assert_eq!(
        stash("stash_1".to_owned()),
        StashReference::Id("stash_1".to_owned())
    );
}

#[tokio::test]
async fn accepted_runs_print_the_accepted_status() {
    let ran = run(&run_args(&[]), ran_with(Err(Error::Accepted))).await;
    assert_eq!(ran.exit, Exit::Success);
    assert_eq!(ran.stdout, "{\n  \"status\": \"accepted\"\n}\n");
}

#[tokio::test]
async fn failed_runs_exit_seven_without_provider_text() {
    let error = serde_json::from_value(json!({
        "attribution": {"origin": "upstream_api", "last_call": {"status": 401}},
        "name": "UpstreamError",
        "message": "token xoxb-hidden expired",
    }))
    .unwrap();
    let failed = Error::ExecutionFailed {
        status: 400,
        error: Box::new(error),
    };
    let ran = run(&run_args(&[]), ran_with(Err(failed))).await;
    assert_eq!(ran.exit, Exit::Rejected);
    assert_eq!(
        ran.stderr,
        "error: The action failed at the app API with status 401. Check the configured props, \
         then run the command again.\n"
    );
    assert!(!ran.stderr.contains("xoxb-hidden"));
}

#[tokio::test]
async fn unavailable_runs_warn_that_the_action_may_have_run() {
    let ran = run(
        &run_args(&[]),
        ran_with(Err(Error::Unavailable {
            status: None,
            retry_after: None,
        })),
    )
    .await;
    assert_eq!(ran.exit, Exit::Unavailable);
    assert_eq!(
        ran.stderr,
        "error: The CLI could not reach Pipedream at https://api.pipedream.com. The action \
         may have run. Check its effects before you run it again.\n"
    );
}

#[tokio::test]
async fn unauthorized_runs_exit_three() {
    let ran = run(
        &run_args(&[]),
        ran_with(Err(Error::Unauthorized { message: None })),
    )
    .await;
    assert_eq!(ran.exit, Exit::Authentication);
}

#[tokio::test]
async fn invalid_run_input_fails_before_any_request() {
    let unreadable = (
        Env::CONFIGURED.clauses(),
        ReadInputMock
            .next_call(matching!(_))
            .answers(&|_, _| Err(io::Error::from(io::ErrorKind::NotFound))),
    );
    let ran = run(&run_args(&["--props", "missing.json"]), unreadable).await;
    assert_eq!(ran.exit, Exit::Usage);
    assert_eq!(
        ran.stderr,
        "error: The --props file does not exist. Check the path.\n"
    );

    let invalid = (
        Env::CONFIGURED.clauses(),
        ReadInputMock
            .next_call(matching!(_))
            .returns(Ok(br#"{"calendarId": "#.to_vec())),
    );
    let ran = run(&run_args(&["--props", "-"]), invalid).await;
    assert_eq!(ran.exit, Exit::Usage);
    assert_eq!(
        ran.stderr,
        "error: --props is not valid JSON at line 1, column 15. Fix the JSON, then run the \
         command again.\n"
    );

    let ran = run(
        &[
            "actions",
            "run",
            "google_calendar-list-events",
            "--external-user-id",
            "user/1",
        ],
        Env::CONFIGURED.clauses(),
    )
    .await;
    assert_eq!(ran.exit, Exit::Usage);
    assert_eq!(
        ran.stderr,
        "error: --external-user-id is not a valid external user ID. Use 1 to 250 visible \
         ASCII characters without spaces, /, \\, ?, or #. The values . and .. are not \
         valid.\n"
    );
}

#[tokio::test]
async fn a_closed_stdout_after_a_run_says_that_the_action_ran() {
    for result in [Ok(response()), Err(Error::Accepted)] {
        let ran = run_closed(&run_args(&[]), ran_with(result)).await;
        assert_eq!(ran.exit, Exit::Internal);
        assert_eq!(
            ran.stderr,
            "error: The CLI cannot write to standard output. Pipedream ran the action or \
             accepted it for later completion. Check its effects before you run it again.\n"
        );
    }
}
