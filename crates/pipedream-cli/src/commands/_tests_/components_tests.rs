//! Tests for `components list` and `components get` against a mocked trait object.

use std::io;

use pipedream_connect::error::Error;
use pipedream_connect::resources::components::ComponentsApiMock;
use pipedream_connect_types::{
    AppSlug, Component, ComponentType, Cursor, Data, ListComponentsRequest, Page, PageInfo,
    PageRequest, RegistryKind, UnknownFields,
};
use serde_json::json;
use unimock::{MockFn, matching};

use crate::report::exit::Exit;
use crate::test_support::{Env, components_handle, connected, printed, run};

fn component() -> Component {
    serde_json::from_value(json!({
        "key": "google_calendar-new-event",
        "name": "New event",
        "version": "0.1.2",
        "component_type": "trigger",
        "configurable_props": [
            {"name": "googleCalendar", "type": "app", "app": "google_calendar"},
            {"name": "calendarId", "type": "string", "remoteOptions": true},
        ],
    }))
    .unwrap()
}

fn page() -> Page<Component> {
    Page {
        data: vec![component()],
        page_info: PageInfo {
            count: Some(1),
            ..PageInfo::default()
        },
        extra: UnknownFields::new(),
    }
}

fn filtered_request() -> ListComponentsRequest {
    ListComponentsRequest {
        page: PageRequest {
            limit: 50,
            after: None,
            before: Some(Cursor::new("cursor_0").unwrap()),
        },
        query: Some("event".to_owned()),
        app: Some(AppSlug::new("google_calendar").unwrap()),
        registry: Some(RegistryKind::Public),
        component_type: Some(ComponentType::Trigger),
    }
}

#[tokio::test]
async fn components_list_maps_every_filter_and_prints_the_page() {
    let clauses = (
        connected(),
        components_handle(),
        ComponentsApiMock::list
            .next_call(matching!(eq!(&filtered_request())))
            .returns(Ok(page())),
    );
    let ran = run(
        &[
            "components",
            "list",
            "--query",
            "event",
            "--app",
            "google_calendar",
            "--registry",
            "public",
            "--type",
            "trigger",
            "--before",
            "cursor_0",
            "--limit",
            "50",
        ],
        clauses,
    )
    .await;
    assert_eq!(ran.exit, Exit::Success);
    assert_eq!(ran.stdout, printed(&page()));
}

#[tokio::test]
async fn components_get_sends_the_key_and_the_version() {
    let response = Data {
        data: component(),
        extra: UnknownFields::new(),
    };
    let clauses = (
        connected(),
        components_handle(),
        ComponentsApiMock::get
            .next_call(
                matching!((key, request) if key.as_str() == "google_calendar-new-event"
                && request.version.as_deref() == Some("0.1.2")),
            )
            .returns(Ok(response.clone())),
    );
    let ran = run(
        &[
            "components",
            "get",
            "google_calendar-new-event",
            "--version",
            "0.1.2",
        ],
        clauses,
    )
    .await;
    assert_eq!(ran.exit, Exit::Success);
    assert_eq!(ran.stdout, printed(&response));
}

#[tokio::test]
async fn components_get_without_a_version_asks_for_the_latest() {
    let clauses = (
        connected(),
        components_handle(),
        ComponentsApiMock::get
            .next_call(matching!((_, request) if request.version.is_none()))
            .returns(Err(Error::Unavailable {
                status: Some(503),
                retry_after: None,
            })),
    );
    let ran = run(&["components", "get", "google_calendar-new-event"], clauses).await;
    assert_eq!(ran.exit, Exit::Unavailable);
    assert_eq!(
        ran.stderr,
        "error: Pipedream at https://api.pipedream.com is unavailable with status 503. Try \
         again later.\n"
    );
}

#[tokio::test]
async fn unreadable_responses_exit_one() {
    let clauses = (
        connected(),
        components_handle(),
        ComponentsApiMock::list
            .next_call(matching!(_))
            .returns(Err(Error::internal(io::Error::other("decode at 1:2")))),
    );
    let ran = run(&["components", "list"], clauses).await;
    assert_eq!(ran.exit, Exit::Internal);
    assert_eq!(
        ran.stderr,
        "error: Pipedream returned a response that the CLI cannot read. Report this problem \
         with the command that you ran at \
         https://github.com/futex-ai/pipedream-connectors/issues.\n"
    );
}

#[tokio::test]
async fn components_commands_reject_invalid_input_before_any_request() {
    let cases = [
        (
            &["components", "list", "--app", "google/calendar"][..],
            "error: --app is not a valid app slug. Use 1 to 250 visible ASCII characters \
             without spaces, /, \\, ?, or #. The values . and .. are not valid.\n",
        ),
        (
            &["components", "get", "list?events"],
            "error: <COMPONENT> is not a valid component key. Use 1 to 250 visible ASCII \
             characters without spaces, /, \\, ?, or #. The values . and .. are not valid.\n",
        ),
    ];
    for (args, stderr) in cases {
        let ran = run(args, Env::CONFIGURED.clauses()).await;
        assert_eq!(ran.exit, Exit::Usage, "{args:?}");
        assert_eq!(ran.stderr, stderr, "{args:?}");
    }
}
