//! Tests for `project info`, `usage`, and `webhook get`.

use pipedream_connect::error::Error;
use pipedream_connect::resources::project_webhooks::ProjectWebhooksApiMock;
use pipedream_connect::resources::projects::ProjectsApiMock;
use pipedream_connect::resources::usage::UsageApiMock;
use pipedream_connect_types::{
    ConnectUsageResponse, GetWebhookResponse, ListUsageRequest, ProjectInfo,
};
use serde_json::json;
use unimock::{MockFn, matching};

use crate::report::exit::Exit;
use crate::test_support::{
    Env, connected, printed, projects_handle, run, usage_handle, webhooks_handle,
};

/// 2026-10-21T00:00:00Z in Unix seconds.
const OCTOBER_21: i64 = 1_792_540_800;

#[tokio::test]
async fn project_info_prints_the_enabled_apps() {
    let info: ProjectInfo = serde_json::from_value(json!({
        "apps": [{"id": "app_1", "name_slug": "google_calendar"}],
    }))
    .unwrap();
    let clauses = (
        connected(),
        projects_handle(),
        ProjectsApiMock::info
            .next_call(matching!())
            .returns(Ok(info.clone())),
    );
    let ran = run(&["project", "info"], clauses).await;
    assert_eq!(ran.exit, Exit::Success);
    assert_eq!(ran.stdout, printed(&info));
}

#[tokio::test]
async fn usage_reads_unix_seconds_and_rfc_3339_times() {
    let records: ConnectUsageResponse = serde_json::from_value(json!({
        "data": [{"credits_used": 12, "usage_start_ts": 1_700_000_000, "usage_end_ts": OCTOBER_21}],
    }))
    .unwrap();
    let clauses = (
        connected(),
        usage_handle(),
        UsageApiMock::list
            .next_call(matching!(eq!(&ListUsageRequest {
                start_ts: 1_700_000_000,
                end_ts: OCTOBER_21,
            })))
            .returns(Ok(records.clone())),
    );
    let args = [
        "usage",
        "--start",
        "1700000000",
        "--end",
        "2026-10-21T02:00:00+02:00",
    ];
    let ran = run(&args, clauses).await;
    assert_eq!(ran.exit, Exit::Success, "{}", ran.stderr);
    assert_eq!(ran.stdout, printed(&records));
}

#[tokio::test]
async fn usage_rejects_a_time_that_is_neither_form() {
    let ran = run(
        &["usage", "--start", "yesterday", "--end", "1700000000"],
        Env::CONFIGURED.clauses(),
    )
    .await;
    assert_eq!(ran.exit, Exit::Usage);
    assert_eq!(
        ran.stderr,
        "error: --start must be Unix seconds or an RFC 3339 time, such as \
         2026-10-01T00:00:00Z.\n"
    );
    let missing = run(&["usage", "--start", "1700000000"], ()).await;
    assert_eq!(missing.exit, Exit::Usage);
}

#[tokio::test]
async fn webhook_get_prints_the_webhook_or_null() {
    let webhook: GetWebhookResponse = serde_json::from_value(json!({
        "data": {"id": "hook_1", "url": "https://example.com/hook", "signing_key_set": true,
                 "created_at": 1_700_000_000, "updated_at": 1_700_000_000},
    }))
    .unwrap();
    let none: GetWebhookResponse = serde_json::from_value(json!({"data": null})).unwrap();
    for response in [webhook, none] {
        let clauses = (
            connected(),
            webhooks_handle(),
            ProjectWebhooksApiMock::get
                .next_call(matching!())
                .returns(Ok(response.clone())),
        );
        let ran = run(&["webhook", "get"], clauses).await;
        assert_eq!(ran.exit, Exit::Success);
        assert_eq!(ran.stdout, printed(&response));
    }
}

#[tokio::test]
async fn admin_reads_report_a_missing_project() {
    let clauses = (
        connected(),
        projects_handle(),
        ProjectsApiMock::info
            .next_call(matching!())
            .returns(Err(Error::NotFound { message: None })),
    );
    let ran = run(&["project", "info"], clauses).await;
    assert_eq!(ran.exit, Exit::NotFound);
    assert_eq!(
        ran.stderr,
        "error: Pipedream did not find the project. Check the project ID and the environment.\n"
    );
}
