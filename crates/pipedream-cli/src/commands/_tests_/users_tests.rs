//! Tests for `users list` against a mocked user trait object.

use pipedream_connect::error::Error;
use pipedream_connect::resources::users::UsersApiMock;
use pipedream_connect_types::{Cursor, ExternalUser, Page, PageInfo, UnknownFields};
use serde_json::{Value, json};
use unimock::{MockFn, matching};

use crate::report::exit::Exit;
use crate::test_support::{connected, printed, run, users_handle};

fn user(id: &str) -> ExternalUser {
    serde_json::from_value(json!({"id": format!("exu_{id}"), "external_id": id})).unwrap()
}

fn page(ids: &[&str], end_cursor: Option<&str>) -> Page<ExternalUser> {
    Page {
        data: ids.iter().map(|id| user(id)).collect(),
        page_info: PageInfo {
            end_cursor: end_cursor.map(|cursor| Cursor::new(cursor).unwrap()),
            ..PageInfo::default()
        },
        extra: UnknownFields::new(),
    }
}

#[tokio::test]
async fn users_list_maps_the_query_and_prints_the_page() {
    let clauses = (
        connected(),
        users_handle(),
        UsersApiMock::list
            .next_call(
                matching!((request) if request.query.as_deref() == Some("alice")
                && request.page.limit == 100),
            )
            .returns(Ok(page(&["alice"], None))),
    );
    let ran = run(&["users", "list", "--query", "alice"], clauses).await;
    assert_eq!(ran.exit, Exit::Success);
    assert_eq!(ran.stdout, printed(&page(&["alice"], None)));
}

#[tokio::test]
async fn users_list_all_reads_every_page() {
    let clauses = (
        connected(),
        users_handle(),
        UsersApiMock::list
            .next_call(matching!((request) if request.page.after.is_none()))
            .returns(Ok(page(&["alice"], Some("cursor_1")))),
        UsersApiMock::list
            .next_call(
                matching!((request) if request.page.after.as_ref().map(Cursor::as_str)
                == Some("cursor_1")),
            )
            .returns(Ok(page(&["bob"], None))),
    );
    let ran = run(&["users", "list", "--all"], clauses).await;
    assert_eq!(ran.exit, Exit::Success);
    let printed: Value = serde_json::from_str(&ran.stdout).unwrap();
    assert_eq!(
        printed,
        json!({"data": [user("alice"), user("bob")], "pages": 2, "end": "exhausted", "next": null})
    );
}

#[tokio::test]
async fn users_list_rate_limit_exits_five() {
    let clauses = (
        connected(),
        users_handle(),
        UsersApiMock::list
            .next_call(matching!(_))
            .returns(Err(Error::RateLimited {
                retry_after: None,
                limit: None,
                remaining: None,
                reset: None,
            })),
    );
    let ran = run(&["users", "list"], clauses).await;
    assert_eq!(ran.exit, Exit::RateLimited);
}
