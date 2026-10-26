//! Request bounds, query encoding, failed token admission, and runtime injection.

use std::num::NonZeroU32;
use std::task::{Context, Poll, Waker};
use std::time::Duration;
use std::{future::Future, pin::pin};

use http::{HeaderValue, Method};
use pipedream_connect_types::{Cursor, PageRequest};
use serde::Serialize;
use serde_json::json;

use crate::config::HttpOptions;
use crate::error::Error;
use crate::request::{ApiPath, ApiRequest};
use crate::retry::RetryPolicy;
use crate::validation::Validate;

use super::support::{Harness, config, response, static_config, token};

#[tokio::test]
async fn the_configured_timeout_reaches_both_token_and_connect_requests() {
    let harness = Harness::script(
        config().with_http_options(HttpOptions {
            request_timeout: Duration::from_secs(42),
            ..HttpOptions::default()
        }),
        vec![Ok(token("bearer", 3600)), Ok(response(200, json!({})))],
    );
    harness.get().await.unwrap();
    assert_eq!(harness.request(0).timeout, Duration::from_secs(42));
    assert_eq!(harness.request(1).timeout, Duration::from_secs(42));
    harness.assert_requests(2);
}

#[tokio::test]
async fn token_endpoint_statuses_are_typed_and_a_later_call_can_refresh() {
    for status in [401, 429, 500, 503] {
        let mut reply = response(status, json!({"error":"secret"}));
        reply
            .headers
            .insert("retry-after", HeaderValue::from_static("17"));
        let harness = Harness::script(
            config().with_retry_policy(RetryPolicy::new(
                NonZeroU32::new(3).unwrap(),
                Duration::from_secs(30),
                Duration::ZERO,
            )),
            vec![
                Ok(reply),
                Ok(token("fresh", 3600)),
                Ok(response(200, json!({}))),
            ],
        );
        let error = harness.get().await.unwrap_err();
        match status {
            401 => assert!(matches!(error, Error::Unauthorized { .. })),
            429 => assert!(
                matches!(error, Error::RateLimited { retry_after: Some(delay), .. } if delay == Duration::from_secs(17))
            ),
            _ => assert!(
                matches!(error, Error::Unavailable { status: Some(actual), .. } if actual == status)
            ),
        }
        assert_eq!(harness.requests.lock().unwrap().len(), 1);
        harness.get().await.unwrap();
        assert_eq!(harness.request(2).headers["authorization"], "Bearer fresh");
        harness.assert_requests(3);
    }
}

#[tokio::test]
async fn page_queries_keep_limit_and_each_cursor_direction() {
    for (after, before, expected) in [
        (Some("next+cursor"), None, "after=next%2Bcursor&limit=23"),
        (
            None,
            Some("previous&cursor"),
            "before=previous%26cursor&limit=23",
        ),
    ] {
        let harness = Harness::script(static_config(), vec![Ok(response(200, json!({})))]);
        let request = ApiRequest::new(Method::GET, ApiPath::connect(&["apps"]))
            .with_page(&PageRequest {
                limit: 23,
                after: after.map(|value| Cursor::new(value).unwrap()),
                before: before.map(|value| Cursor::new(value).unwrap()),
            })
            .unwrap();
        harness.client.dispatcher().dispatch(request).await.unwrap();
        assert_eq!(
            harness.request(0).url.expose_secret(),
            format!("https://api.pipedream.com/v1/connect/apps?{expected}")
        );
        harness.assert_requests(1);
    }
}

#[derive(Serialize)]
struct NestedQuery {
    filter: NestedFilter,
}

#[derive(Serialize)]
struct NestedFilter {
    value: String,
}

impl Validate for NestedQuery {
    fn validate(&self) -> crate::error::Result<()> {
        Ok(())
    }
}

#[test]
fn query_encoding_rejects_nested_objects() {
    let error = ApiRequest::new(Method::GET, ApiPath::connect(&["apps"]))
        .with_query(&NestedQuery {
            filter: NestedFilter {
                value: "value".into(),
            },
        })
        .unwrap_err();
    assert!(matches!(error, Error::Internal(_)));
}

#[tokio::test]
async fn oversize_errors_precede_status_mapping_and_retry() {
    for status in [401, 429, 503] {
        for announced in [false, true] {
            let mut reply = response(status, json!({}));
            if announced {
                reply
                    .headers
                    .insert("content-length", HeaderValue::from_static("3"));
            } else {
                reply.body.push(b' ');
            }
            let harness = Harness::script(
                static_config()
                    .with_http_options(HttpOptions {
                        response_limit: 2,
                        ..HttpOptions::default()
                    })
                    .with_retry_policy(RetryPolicy::new(
                        NonZeroU32::new(3).unwrap(),
                        Duration::from_secs(30),
                        Duration::ZERO,
                    )),
                vec![Ok(reply)],
            );
            assert!(matches!(
                harness.get().await,
                Err(Error::ResponseTooLarge { status: actual, limit: 2 }) if actual == status
            ));
            harness.assert_requests(1);
        }
    }
}

#[test]
fn an_injected_sleeper_allows_get_retries_without_a_tokio_runtime() {
    let harness = Harness::script(
        static_config().with_retry_policy(RetryPolicy::new(
            NonZeroU32::new(2).unwrap(),
            Duration::from_secs(30),
            Duration::from_secs(2),
        )),
        vec![Ok(response(503, json!({}))), Ok(response(200, json!({})))],
    )
    .record_sleeps();
    let mut future = pin!(harness.get());
    let result = future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()));
    assert!(matches!(result, Poll::Ready(Ok(reply)) if reply.status == 200));
    harness.assert_requests(2);
    assert_eq!(*harness.sleeps.lock().unwrap(), [Duration::from_secs(2)]);
}
