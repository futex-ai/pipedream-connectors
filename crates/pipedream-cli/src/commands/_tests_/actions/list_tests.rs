//! Tests for `actions list`, including bounded traversal with `--all`.

use pipedream_connect::error::Error;
use pipedream_connect::resources::actions::ActionsApiMock;
use pipedream_connect_types::{
    AppSlug, Component, Cursor, ListRegistryRequest, Page, PageInfo, RegistryKind, UnknownFields,
};
use serde_json::{Value, json};
use unimock::{MockFn, matching};

use crate::report::exit::Exit;
use crate::test_support::{actions_handle, connected, printed, run};

fn action(key: &str) -> Component {
    serde_json::from_value(json!({
        "key": key,
        "name": key,
        "version": "1.0.0",
        "component_type": "action",
        "configurable_props": [],
    }))
    .unwrap()
}

fn page(keys: &[&str], end_cursor: Option<&str>) -> Page<Component> {
    Page {
        data: keys.iter().map(|key| action(key)).collect(),
        page_info: PageInfo {
            end_cursor: end_cursor.map(|cursor| Cursor::new(cursor).unwrap()),
            ..PageInfo::default()
        },
        extra: UnknownFields::new(),
    }
}

fn filtered_request() -> ListRegistryRequest {
    ListRegistryRequest {
        query: Some("message".to_owned()),
        app: Some(AppSlug::new("slack").unwrap()),
        registry: Some(RegistryKind::All),
        ..ListRegistryRequest::default()
    }
}

fn after(request: &ListRegistryRequest) -> Option<&str> {
    request.page.after.as_ref().map(Cursor::as_str)
}

#[tokio::test]
async fn actions_list_maps_the_filters_and_prints_the_page() {
    let clauses = (
        connected(),
        actions_handle(),
        ActionsApiMock::list
            .next_call(matching!(eq!(&filtered_request())))
            .returns(Ok(page(&["slack-send-message"], None))),
    );
    let ran = run(
        &[
            "actions",
            "list",
            "--app",
            "slack",
            "--registry",
            "all",
            "--query",
            "message",
        ],
        clauses,
    )
    .await;
    assert_eq!(ran.exit, Exit::Success);
    assert_eq!(ran.stdout, printed(&page(&["slack-send-message"], None)));
}

#[tokio::test]
async fn actions_list_all_stops_at_the_page_budget_with_a_resume_cursor() {
    let clauses = (
        connected(),
        actions_handle(),
        ActionsApiMock::list
            .next_call(matching!((request) if after(request).is_none() && request.page.limit == 1))
            .returns(Ok(page(&["first"], Some("cursor_1")))),
        ActionsApiMock::list
            .next_call(matching!((request) if after(request) == Some("cursor_1")))
            .returns(Ok(page(&["second"], Some("cursor_2")))),
    );
    let ran = run(
        &[
            "actions",
            "list",
            "--limit",
            "1",
            "--all",
            "--max-pages",
            "2",
        ],
        clauses,
    )
    .await;
    assert_eq!(ran.exit, Exit::Success);
    let printed: Value = serde_json::from_str(&ran.stdout).unwrap();
    assert_eq!(
        printed,
        json!({
            "data": [action("first"), action("second")],
            "pages": 2,
            "end": "page_limit",
            "next": {"limit": 1, "after": "cursor_2"},
        })
    );
    assert_eq!(
        ran.stderr,
        "note: The CLI read 2 pages and more records exist. To continue, run the command again \
         with --after cursor_2.\n"
    );
}

#[tokio::test]
async fn actions_list_all_stops_on_a_repeated_cursor() {
    let clauses = (
        connected(),
        actions_handle(),
        ActionsApiMock::list
            .next_call(matching!((request) if after(request) == Some("cursor_1")))
            .returns(Ok(page(&["repeat"], Some("cursor_1")))),
    );
    let ran = run(
        &["actions", "list", "--after", "cursor_1", "--all"],
        clauses,
    )
    .await;
    assert_eq!(ran.exit, Exit::Success);
    let printed: Value = serde_json::from_str(&ran.stdout).unwrap();
    assert_eq!(printed["end"], json!("stuck_cursor"));
    assert_eq!(printed["pages"], json!(1));
    assert_eq!(printed["next"], Value::Null);
    assert_eq!(
        ran.stderr,
        "note: Pipedream repeated a page cursor. The result holds the records before the \
         repeat.\n"
    );
}

#[tokio::test]
async fn actions_list_all_names_the_resume_cursor_after_a_failure() {
    let clauses = (
        connected(),
        actions_handle(),
        ActionsApiMock::list
            .next_call(matching!((request) if after(request).is_none()))
            .returns(Ok(page(&["first"], Some("cursor_1")))),
        ActionsApiMock::list
            .next_call(matching!((request) if after(request) == Some("cursor_1")))
            .returns(Err(Error::Unavailable { status: None })),
    );
    let ran = run(&["actions", "list", "--all"], clauses).await;
    assert_eq!(ran.exit, Exit::Unavailable);
    assert!(ran.stdout.is_empty());
    assert_eq!(
        ran.stderr,
        "error: Pipedream did not respond. Try again later. The CLI read 1 page before the \
         failure. To continue, run the command again with --after cursor_1.\n"
    );
}

#[tokio::test]
async fn actions_list_rejected_exits_seven() {
    let clauses = (
        connected(),
        actions_handle(),
        ActionsApiMock::list
            .next_call(matching!(_))
            .returns(Err(Error::Rejected {
                status: 400,
                code: None,
                message: Some("bad registry".to_owned()),
            })),
    );
    let ran = run(&["actions", "list"], clauses).await;
    assert_eq!(ran.exit, Exit::Rejected);
    assert_eq!(
        ran.stderr,
        "error: Pipedream rejected the request with status 400. Check the command arguments.\n"
    );
}
