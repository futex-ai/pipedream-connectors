//! Explicit retry delays and attempt counts, with no real sleep.

use std::num::NonZeroU32;
use std::time::{Duration, UNIX_EPOCH};

use http::{HeaderValue, Method};
use serde_json::json;

use crate::error::Error;
use crate::request::{ApiPath, ApiRequest};
use crate::retry::RetryPolicy;

use super::support::{Harness, response, static_config};

fn policy(attempts: u32) -> RetryPolicy {
    RetryPolicy::new(
        NonZeroU32::new(attempts).unwrap(),
        Duration::from_secs(30),
        Duration::from_secs(2),
    )
}

#[tokio::test]
async fn retry_after_dates_seconds_and_fallback_delays_obey_the_cap() {
    for (header, delay) in [
        (Some("4".to_owned()), 4),
        (Some("60".to_owned()), 30),
        (
            Some(httpdate::fmt_http_date(
                UNIX_EPOCH + Duration::from_secs(1012),
            )),
            12,
        ),
        (Some("malformed".to_owned()), 2),
        (None, 2),
    ] {
        let mut reply = response(429, json!({}));
        if let Some(header) = header {
            reply
                .headers
                .insert("retry-after", HeaderValue::from_str(&header).unwrap());
        }
        let harness = Harness::script(
            static_config().with_retry_policy(policy(3)),
            vec![Ok(reply), Ok(response(200, json!({})))],
        )
        .record_sleeps();
        harness.get().await.unwrap();
        harness.assert_requests(2);
        assert_eq!(
            *harness.sleeps.lock().unwrap(),
            [Duration::from_secs(delay)]
        );
    }
}

#[tokio::test]
async fn retryable_statuses_stop_at_the_total_attempt_budget() {
    for status in [408, 429, 500, 502, 503, 599] {
        let harness = Harness::script(
            static_config().with_retry_policy(policy(3)),
            (0..3).map(|_| Ok(response(status, json!({})))).collect(),
        )
        .record_sleeps();
        let result = harness.get().await;
        assert!(matches!(
            result,
            Err(Error::Unavailable { .. }) | Err(Error::RateLimited { .. })
        ));
        harness.assert_requests(3);
        assert_eq!(*harness.sleeps.lock().unwrap(), [Duration::from_secs(2); 2]);
        let requests = harness.requests.lock().unwrap();
        assert!(requests.iter().all(|request| request.method == Method::GET));
        assert_eq!(requests[0].url, requests[1].url);
        assert_eq!(requests[0].headers, requests[1].headers);
    }
}

#[tokio::test]
async fn mutations_and_nonretryable_statuses_are_never_replayed() {
    for method in [
        Method::POST,
        Method::PUT,
        Method::PATCH,
        Method::DELETE,
        Method::HEAD,
    ] {
        for status in [408, 429, 500] {
            let harness = Harness::script(
                static_config().with_retry_policy(policy(3)),
                vec![Ok(response(status, json!({})))],
            );
            let request = ApiRequest::new(method.clone(), ApiPath::project(&["actions", "run"]));
            assert!(harness.client.dispatcher().dispatch(request).await.is_err());
            harness.assert_requests(1);
            assert!(harness.sleeps.lock().unwrap().is_empty());
        }
    }
    for status in [301, 400, 401, 403, 404, 409, 422] {
        let harness = Harness::script(
            static_config().with_retry_policy(policy(3)),
            vec![Ok(response(status, json!({})))],
        );
        assert!(harness.get().await.is_err());
        harness.assert_requests(1);
        assert!(harness.sleeps.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn no_policy_one_attempt_and_transport_errors_have_no_retry() {
    for config in [
        static_config(),
        static_config().with_retry_policy(policy(1)),
    ] {
        let harness = Harness::script(config, vec![Ok(response(503, json!({})))]);
        assert!(harness.get().await.is_err());
        harness.assert_requests(1);
        assert!(harness.sleeps.lock().unwrap().is_empty());
    }
    let harness = Harness::script(
        static_config().with_retry_policy(policy(3)),
        vec![Err(Error::Unavailable { status: None })],
    );
    assert!(matches!(
        harness.get().await,
        Err(Error::Unavailable { status: None })
    ));
    harness.assert_requests(1);
    assert!(harness.sleeps.lock().unwrap().is_empty());
}
