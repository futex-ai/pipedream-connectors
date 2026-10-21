//! Tests for `apps list` and `apps get` against a mocked app trait object.

use pipedream_connect::error::Error;
use pipedream_connect::resources::apps::AppsApiMock;
use pipedream_connect_types::{
    App, AppCategoryId, AppSortKey, Cursor, Data, ListAppsRequest, Page, PageInfo, PageRequest,
    SortDirection, UnknownFields,
};
use serde_json::json;
use unimock::{MockFn, matching};

use crate::report::exit::Exit;
use crate::test_support::{Env, apps_handle, connected, printed, run};

fn app(slug: &str) -> App {
    serde_json::from_value(json!({
        "id": format!("app_{slug}"),
        "name_slug": slug,
        "name": slug,
        "auth_type": "oauth",
        "description": null,
        "img_src": "https://example.com/logo.png",
        "custom_fields_json": null,
    }))
    .unwrap()
}

fn page(apps: Vec<App>, end_cursor: Option<&str>) -> Page<App> {
    Page {
        page_info: PageInfo {
            count: Some(apps.len() as u64),
            end_cursor: end_cursor.map(|cursor| Cursor::new(cursor).unwrap()),
            ..PageInfo::default()
        },
        data: apps,
        extra: UnknownFields::new(),
    }
}

fn filtered_request() -> ListAppsRequest {
    ListAppsRequest {
        page: PageRequest {
            limit: 25,
            after: Some(Cursor::new("cursor_1").unwrap()),
            before: None,
        },
        query: Some("calendar".to_owned()),
        sort_key: Some(AppSortKey::FeaturedWeight),
        sort_direction: Some(SortDirection::Desc),
        category_ids: vec![
            AppCategoryId::new("appcat_Productivity").unwrap(),
            AppCategoryId::new("appcat_Calendar").unwrap(),
        ],
        has_components: None,
        has_actions: Some(true),
        has_triggers: Some(false),
    }
}

#[tokio::test]
async fn apps_list_maps_every_filter_and_prints_the_page() {
    let clauses = (
        connected(),
        apps_handle(),
        AppsApiMock::list
            .next_call(matching!(eq!(&filtered_request())))
            .returns(Ok(page(vec![app("google_calendar")], Some("cursor_2")))),
    );
    let ran = run(
        &[
            "apps",
            "list",
            "--query",
            "calendar",
            "--sort-key",
            "featured_weight",
            "--sort-direction",
            "desc",
            "--category",
            "appcat_Productivity",
            "--category",
            "appcat_Calendar",
            "--has-actions",
            "--has-triggers",
            "false",
            "--limit",
            "25",
            "--after",
            "cursor_1",
        ],
        clauses,
    )
    .await;
    assert_eq!(ran.exit, Exit::Success);
    assert_eq!(
        ran.stdout,
        printed(&page(vec![app("google_calendar")], Some("cursor_2")))
    );
    assert!(ran.stderr.is_empty());
}

#[tokio::test]
async fn apps_list_without_flags_requests_the_default_page() {
    let clauses = (
        connected(),
        apps_handle(),
        AppsApiMock::list
            .next_call(matching!(eq!(&ListAppsRequest::default())))
            .returns(Ok(page(Vec::new(), None))),
    );
    let ran = run(&["apps", "list"], clauses).await;
    assert_eq!(ran.exit, Exit::Success);
    assert_eq!(ran.stdout, printed(&page(Vec::new(), None)));
}

#[tokio::test]
async fn apps_list_all_reads_every_page_until_the_last() {
    let clauses = (
        connected(),
        apps_handle(),
        AppsApiMock::list
            .next_call(matching!((request) if request.page.limit == 1 && request.page.after.is_none()
                && request.query.as_deref() == Some("mail")))
            .returns(Ok(page(vec![app("gmail")], Some("cursor_1")))),
        AppsApiMock::list
            .next_call(matching!((request) if request.page.after.as_ref().map(Cursor::as_str) == Some("cursor_1")
                && request.query.as_deref() == Some("mail")))
            .returns(Ok(page(vec![app("outlook")], None))),
    );
    let ran = run(
        &["apps", "list", "--query", "mail", "--limit", "1", "--all"],
        clauses,
    )
    .await;
    assert_eq!(ran.exit, Exit::Success);
    let printed: serde_json::Value = serde_json::from_str(&ran.stdout).unwrap();
    assert_eq!(
        printed,
        json!({
            "data": [app("gmail"), app("outlook")],
            "pages": 2,
            "end": "exhausted",
            "next": null,
        })
    );
    assert!(ran.stderr.is_empty());
}

#[tokio::test]
async fn apps_get_reads_one_app_by_slug() {
    let response = Data {
        data: app("google_calendar"),
        extra: UnknownFields::new(),
    };
    let clauses = (
        connected(),
        apps_handle(),
        AppsApiMock::get
            .next_call(matching!((slug) if slug.as_str() == "google_calendar"))
            .returns(Ok(response.clone())),
    );
    let ran = run(&["apps", "get", "google_calendar"], clauses).await;
    assert_eq!(ran.exit, Exit::Success);
    assert_eq!(ran.stdout, printed(&response));
}

#[tokio::test]
async fn apps_get_not_found_exits_four_without_provider_text() {
    let clauses = (
        connected(),
        apps_handle(),
        AppsApiMock::get
            .next_call(matching!(_))
            .returns(Err(Error::NotFound {
                message: Some("no app named missing_app".to_owned()),
            })),
    );
    let ran = run(&["apps", "get", "missing_app"], clauses).await;
    assert_eq!(ran.exit, Exit::NotFound);
    assert!(ran.stdout.is_empty());
    assert_eq!(
        ran.stderr,
        "error: Pipedream did not find the app. Check the app slug or ID.\n"
    );
}

#[tokio::test]
async fn apps_list_forbidden_exits_three() {
    let clauses = (
        connected(),
        apps_handle(),
        AppsApiMock::list
            .next_call(matching!(_))
            .returns(Err(Error::Forbidden { message: None })),
    );
    let ran = run(&["apps", "list"], clauses).await;
    assert_eq!(ran.exit, Exit::Authentication);
    assert!(ran.stderr.starts_with("error: Pipedream denied access."));
}

#[tokio::test]
async fn apps_commands_reject_invalid_input_before_any_request() {
    let cases = [
        (
            &["apps", "list", "--category", "productivity"][..],
            "error: --category is not a valid category ID. Use an ID that starts with appcat_.\n",
        ),
        (
            &["apps", "list", "--limit", "0"],
            "error: --limit must be from 1 to 200.\n",
        ),
        (
            &["apps", "list", "--limit", "201"],
            "error: --limit must be from 1 to 200.\n",
        ),
        (
            &["apps", "list", "--before", "a/b"],
            "error: --before is not a valid cursor. Copy the cursor from the page_info of an \
             earlier result.\n",
        ),
        (
            &["apps", "get", "google/calendar"],
            "error: <APP> is not a valid app slug. Use 1 to 250 visible ASCII characters \
             without /, \\, ?, or #.\n",
        ),
    ];
    for (args, stderr) in cases {
        let ran = run(args, Env::CONFIGURED.clauses()).await;
        assert_eq!(ran.exit, Exit::Usage, "{args:?}");
        assert_eq!(ran.stderr, stderr, "{args:?}");
        assert!(ran.stdout.is_empty());
    }
}
