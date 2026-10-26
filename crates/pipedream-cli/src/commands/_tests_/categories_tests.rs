//! Tests for `categories list` against a mocked category trait object.

use std::time::Duration;

use pipedream_connect::error::Error;
use pipedream_connect::resources::app_categories::AppCategoriesApiMock;
use pipedream_connect_types::AppCategory;
use serde_json::json;
use unimock::{MockFn, matching};

use crate::report::exit::Exit;
use crate::test_support::{categories_handle, connected, printed, run};

fn categories() -> Vec<AppCategory> {
    serde_json::from_value(json!([
        {"id": "appcat_Productivity", "name": "Productivity", "description": null},
        {"id": "appcat_Calendar", "name": "Calendar", "description": "Scheduling apps"},
    ]))
    .unwrap()
}

#[tokio::test]
async fn categories_list_prints_the_bare_array() {
    let clauses = (
        connected(),
        categories_handle(),
        AppCategoriesApiMock::list
            .next_call(matching!())
            .returns(Ok(categories())),
    );
    let ran = run(&["categories", "list"], clauses).await;
    assert_eq!(ran.exit, Exit::Success);
    assert_eq!(ran.stdout, printed(&categories()));
    assert!(ran.stderr.is_empty());
}

#[tokio::test]
async fn categories_list_has_no_page_flags() {
    let ran = run(&["categories", "list", "--limit", "5"], ()).await;
    assert_eq!(ran.exit, Exit::Usage);
    assert!(ran.stderr.contains("unexpected argument '--limit'"));
}

#[tokio::test]
async fn categories_rate_limit_exits_five_with_the_wait_time() {
    let clauses = (
        connected(),
        categories_handle(),
        AppCategoriesApiMock::list
            .next_call(matching!())
            .returns(Err(Error::RateLimited {
                retry_after: Some(Duration::from_secs(30)),
                reset_after: None,
                limit: Some(60),
                remaining: Some(0),
                reset: Some(1_700_000_030),
            })),
    );
    let ran = run(&["categories", "list"], clauses).await;
    assert_eq!(ran.exit, Exit::RateLimited);
    assert_eq!(
        ran.stderr,
        "error: Pipedream rate limited the request. Wait 30 seconds, then run the command \
         again.\n"
    );
}
