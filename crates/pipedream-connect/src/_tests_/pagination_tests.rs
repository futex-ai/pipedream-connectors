//! Bounded forward and backward traversal with a mocked page source.

use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

use pipedream_connect_types::{Cursor, Page, PageRequest};
use serde_json::json;
use unimock::{MockFn, Unimock, matching};

use crate::error::Error;
use crate::pagination::{
    BoundedPaginator, PageSourceMock, PaginationEnd, PaginationError, Paginator,
};

fn page(data: &[u32], start: Option<&str>, end: Option<&str>) -> Page<u32> {
    serde_json::from_value(
        json!({"data":data,"page_info":{"count":data.len(),"start_cursor":start,"end_cursor":end}}),
    )
    .unwrap()
}

#[tokio::test]
async fn page_limit_returns_a_resumable_cursor_and_records_each_request() {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let recorded = requests.clone();
    let source = Arc::new(Unimock::new(
        PageSourceMock::fetch
            .with_types::<u32>()
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, request| {
                let mut recorded = recorded.lock().unwrap();
                recorded.push(request);
                Ok(if recorded.len() == 1 {
                    page(&[1, 2], Some("first"), Some("second"))
                } else {
                    page(&[3], Some("third"), Some("fourth"))
                })
            })),
    ));
    let paginator: Arc<dyn Paginator<u32> + Send + Sync> = Arc::new(BoundedPaginator::new(source));
    let result = paginator
        .paginate(
            PageRequest::with_limit(2).unwrap(),
            NonZeroUsize::new(2).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(result.data, [1, 2, 3]);
    assert_eq!(result.pages, 2);
    assert_eq!(result.end, PaginationEnd::PageLimit);
    assert_eq!(result.next.unwrap().after.unwrap().as_str(), "fourth");
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert!(requests[0].after.is_none());
    assert_eq!(requests[1].after.as_ref().unwrap().as_str(), "second");
    assert_eq!(requests[1].limit, 2);
}

#[tokio::test]
async fn backward_traversal_stops_on_a_repeated_cursor() {
    let source = Arc::new(Unimock::new((
        PageSourceMock::fetch
            .with_types::<u32>()
            .next_call(matching!((request) if request.before.as_ref().unwrap().as_str() == "third"))
            .returns(Ok(page(&[2], Some("second"), Some("end")))),
        PageSourceMock::fetch
            .with_types::<u32>()
            .next_call(
                matching!((request) if request.before.as_ref().unwrap().as_str() == "second"),
            )
            .returns(Ok(page(&[1], Some("second"), Some("end")))),
    )));
    let paginator: Arc<dyn Paginator<u32> + Send + Sync> = Arc::new(BoundedPaginator::new(source));
    let initial = PageRequest {
        before: Some(Cursor::new("third").unwrap()),
        ..PageRequest::default()
    };
    let result = paginator
        .paginate(initial, NonZeroUsize::new(10).unwrap())
        .await
        .unwrap();
    assert_eq!(result.data, [2, 1]);
    assert_eq!(result.pages, 2);
    assert_eq!(result.end, PaginationEnd::StuckCursor);
    assert!(result.next.is_none());
}

#[tokio::test]
async fn empty_pages_and_absent_cursors_end_traversal() {
    for reply in [
        page(&[], Some("unused"), Some("unused")),
        page(&[1], None, None),
    ] {
        let empty = reply.data.is_empty();
        let source = Arc::new(Unimock::new(
            PageSourceMock::fetch
                .with_types::<u32>()
                .next_call(matching!(_))
                .returns(Ok(reply)),
        ));
        let paginator: Arc<dyn Paginator<u32> + Send + Sync> =
            Arc::new(BoundedPaginator::new(source));
        let result = paginator
            .paginate(PageRequest::default(), NonZeroUsize::new(10).unwrap())
            .await
            .unwrap();
        assert_eq!(result.data.is_empty(), empty);
        assert_eq!(result.pages, 1);
        assert_eq!(result.end, PaginationEnd::Exhausted);
        assert!(result.next.is_none());
    }
}

#[tokio::test]
async fn invalid_initial_pages_send_no_request_and_source_errors_propagate() {
    let paginator: Arc<dyn Paginator<u32> + Send + Sync> =
        Arc::new(BoundedPaginator::new(Arc::new(Unimock::new(()))));
    assert!(matches!(
        paginator
            .paginate(
                PageRequest {
                    limit: 0,
                    ..PageRequest::default()
                },
                NonZeroUsize::new(1).unwrap()
            )
            .await,
        Err(PaginationError::Interrupted {
            source: Error::InvalidRequest(_),
            ..
        })
    ));
    let source = Arc::new(Unimock::new(
        PageSourceMock::fetch
            .with_types::<u32>()
            .next_call(matching!(_))
            .returns(Err(Error::Unavailable { status: Some(503) })),
    ));
    let paginator: Arc<dyn Paginator<u32> + Send + Sync> = Arc::new(BoundedPaginator::new(source));
    assert!(matches!(
        paginator
            .paginate(PageRequest::default(), NonZeroUsize::new(1).unwrap())
            .await,
        Err(PaginationError::Interrupted {
            source: Error::Unavailable { status: Some(503) },
            ..
        })
    ));
}
