//! Failed page recovery and token expiry across wall-clock changes.

use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use pipedream_connect_types::{Page, PageRequest};
use serde_json::json;
use unimock::{MockFn, Unimock, matching};

use crate::error::Error;
use crate::pagination::{
    BoundedPaginator, PageSourceMock, PaginationEnd, PaginationError, Paginator,
};

use super::support::{Harness, config, response, token};

#[tokio::test]
async fn a_page_failure_keeps_prior_records_page_count_and_resume_request() {
    let page: Page<u32> = serde_json::from_value(json!({
        "data":[7,8], "page_info":{"end_cursor":"resume"}
    }))
    .unwrap();
    let source = Arc::new(Unimock::new((
        PageSourceMock::fetch
            .with_types::<u32>()
            .next_call(matching!(_))
            .returns(Ok(page)),
        PageSourceMock::fetch
            .with_types::<u32>()
            .next_call(matching!(_))
            .returns(Err(Error::Unavailable {
                status: Some(503),
                retry_after: None,
            })),
    )));
    let paginator: BoundedPaginator<u32> = BoundedPaginator::new(source);
    let error = paginator
        .paginate(PageRequest::default(), NonZeroUsize::new(3).unwrap())
        .await
        .unwrap_err();
    let debug = format!("{error:?}");
    assert!(debug.contains("pages: 1"), "lost page count: {debug}");
    assert!(debug.contains("resume"), "lost resume request: {debug}");
    assert!(debug.contains("records: 2"), "lost prior records: {debug}");
    let PaginationError::Interrupted {
        data,
        pages,
        request,
        source,
    } = error;
    assert_eq!(data, [7, 8]);
    assert_eq!(pages, 1);
    assert_eq!(request.after.as_ref().unwrap().as_str(), "resume");
    assert_eq!(request.limit, 100);
    assert!(matches!(
        source,
        Error::Unavailable {
            status: Some(503),
            ..
        }
    ));
    let source = Arc::new(Unimock::new(
        PageSourceMock::fetch
            .with_types::<u32>()
            .next_call(matching!((request) if request.after.as_ref().unwrap().as_str() == "resume"))
            .returns(Ok(serde_json::from_value(
                json!({"data":[9],"page_info":{}}),
            )
            .unwrap())),
    ));
    let paginator: BoundedPaginator<u32> = BoundedPaginator::new(source);
    let resumed = paginator
        .paginate(request, NonZeroUsize::new(3).unwrap())
        .await
        .unwrap();
    assert_eq!(resumed.data, [9]);
    assert_eq!(resumed.pages, 1);
    assert_eq!(resumed.end, PaginationEnd::Exhausted);
}

#[tokio::test]
async fn wall_clock_steps_do_not_change_a_token_refresh_deadline() {
    for wall in [0, 10_000_000] {
        let harness = Harness::script(
            config(),
            vec![
                Ok(token("first", 100)),
                Ok(response(200, json!({}))),
                Ok(response(200, json!({}))),
                Ok(token("second", 100)),
                Ok(response(200, json!({}))),
            ],
        );
        harness.get().await.unwrap();
        harness.wall_millis.store(wall, Ordering::SeqCst);
        harness.now_millis.store(1_089_999, Ordering::SeqCst);
        harness.get().await.unwrap();
        assert_eq!(harness.request(2).headers["authorization"], "Bearer first");
        harness.now_millis.store(1_090_000, Ordering::SeqCst);
        harness.get().await.unwrap();
        assert_eq!(harness.request(4).headers["authorization"], "Bearer second");
        harness.assert_requests(5);
    }
}
