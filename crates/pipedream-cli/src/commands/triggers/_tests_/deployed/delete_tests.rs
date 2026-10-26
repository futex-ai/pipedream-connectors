//! Tests for `triggers delete`.

use pipedream_connect::error::Error;
use pipedream_connect::resources::deployed_triggers::DeployedTriggersApiMock;
use unimock::{MockFn, matching};

use crate::report::exit::Exit;
use crate::test_support::{connected, deployed_triggers_handle, run, run_closed};

use super::trigger;

#[tokio::test]
async fn triggers_delete_prints_the_deleted_status() {
    let clauses = (
        connected(),
        deployed_triggers_handle(),
        DeployedTriggersApiMock::delete
            .next_call(matching!((id, request) if id.as_str() == "dc_Trigger1"
                && request.ignore_hook_errors == Some(true)
                && !request.missing_ok))
            .returns(Ok(())),
    );
    let mut args = trigger("delete");
    args.push("--ignore-hook-errors");
    let ran = run(&args, clauses).await;
    assert_eq!(ran.exit, Exit::Success);
    assert_eq!(ran.stdout, "{\n  \"status\": \"deleted\"\n}\n");
}

#[tokio::test]
async fn missing_ok_asks_the_sdk_to_accept_a_missing_trigger() {
    let clauses = (
        connected(),
        deployed_triggers_handle(),
        DeployedTriggersApiMock::delete
            .next_call(matching!((_, request) if request.missing_ok
                && request.ignore_hook_errors.is_none()))
            .returns(Ok(())),
    );
    let mut args = trigger("delete");
    args.push("--missing-ok");
    let ran = run(&args, clauses).await;
    assert_eq!(ran.exit, Exit::Success);
    assert_eq!(ran.stdout, "{\n  \"status\": \"deleted\"\n}\n");
}

#[tokio::test]
async fn a_missing_trigger_without_missing_ok_exits_not_found() {
    let clauses = (
        connected(),
        deployed_triggers_handle(),
        DeployedTriggersApiMock::delete
            .next_call(matching!((_, request) if !request.missing_ok))
            .returns(Err(Error::NotFound { message: None })),
    );
    let ran = run(&trigger("delete"), clauses).await;
    assert_eq!(ran.exit, Exit::NotFound);
    assert!(ran.stdout.is_empty());
    assert_eq!(
        ran.stderr,
        "error: Pipedream did not find the deployed trigger. Check the trigger ID and the \
         external user ID.\n"
    );
}

#[tokio::test]
async fn an_unanswered_delete_warns_that_the_trigger_may_be_gone() {
    let clauses = (
        connected(),
        deployed_triggers_handle(),
        DeployedTriggersApiMock::delete
            .next_call(
                matching!((_, request) if request.ignore_hook_errors.is_none()
                && !request.missing_ok),
            )
            .returns(Err(Error::Unavailable {
                status: None,
                retry_after: None,
            })),
    );
    let ran = run(&trigger("delete"), clauses).await;
    assert_eq!(ran.exit, Exit::Unavailable);
    assert_eq!(
        ran.stderr,
        "error: The CLI could not reach Pipedream at https://api.pipedream.com. Pipedream may \
         have deleted the trigger. Check it with triggers get.\n"
    );
}

#[tokio::test]
async fn an_http_emitter_id_reaches_the_delete_request() {
    let clauses = (
        connected(),
        deployed_triggers_handle(),
        DeployedTriggersApiMock::delete
            .next_call(matching!((id, _) if id.as_str() == "hi_Http1"))
            .returns(Ok(())),
    );
    let args = [
        "triggers",
        "delete",
        "hi_Http1",
        "--external-user-id",
        "user_1",
    ];
    assert_eq!(run(&args, clauses).await.exit, Exit::Success);
}

#[tokio::test]
async fn a_closed_stdout_after_a_delete_says_that_the_trigger_is_deleted() {
    let clauses = (
        connected(),
        deployed_triggers_handle(),
        DeployedTriggersApiMock::delete
            .next_call(matching!(_))
            .returns(Ok(())),
    );
    let ran = run_closed(&trigger("delete"), clauses).await;
    assert_eq!(ran.exit, Exit::Internal);
    assert_eq!(
        ran.stderr,
        "error: The CLI cannot write to standard output. Pipedream deleted the trigger.\n"
    );
}
