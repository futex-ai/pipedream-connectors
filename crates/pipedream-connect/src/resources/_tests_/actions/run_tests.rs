//! Action bodies, stash union branches, nonblocking acceptance, and failures.

use std::num::NonZeroU32;
use std::time::Duration;

use http::HeaderValue;
use pipedream_connect_types::ErrorOrigin;
use serde_json::json;

use crate::error::Error;
use crate::resources::test_support::{assert_post, run};
use crate::retry::RetryPolicy;
use crate::tests::support::{Harness, config, response, token};

#[tokio::test]
async fn a_minimal_run_omits_optional_flags_and_stash_values() {
    let harness = Harness::new(vec![response(200, json!({"ret":null}))]);
    let expected = json!({"id":"google_calendar-list-events","external_user_id":"customer_1"});
    let request = serde_json::from_value(expected.clone()).unwrap();
    harness.client.actions().run(request).await.unwrap();
    assert_post(harness.request(1), "actions/run", &expected);
    harness.assert_requests(2);
}

#[tokio::test]
async fn run_sends_the_exact_body_for_all_blocking_and_stash_variants() {
    for stash in [json!("stash_123"), json!("NEW"), json!(true), json!(false)] {
        let harness = Harness::new(vec![response(
            200,
            json!({"exports":{"summary":"done"},"ret":{"result":42},"os":[],"stash_id":"stash_123","future":{"keep":true}}),
        )]);
        let (request, body) = run(stash);
        let result = harness.client.actions().run(request).await.unwrap();
        assert_post(harness.request(1), "actions/run", &body);
        assert_eq!(result.ret, Some(json!({"result":42})));
        assert_eq!(result.exports, Some(json!({"summary":"done"})));
        assert_eq!(result.stash_id.as_deref(), Some("stash_123"));
        assert_eq!(result.extra["future"], json!({"keep":true}));
        harness.assert_requests(2);
    }
}

#[tokio::test]
async fn nonblocking_202_is_accepted_without_decoding_or_replaying_the_body() {
    for body_bytes in [b"{}".as_slice(), b"".as_slice()] {
        let mut reply = response(202, json!({}));
        reply.body = body_bytes.to_vec();
        let harness = Harness::new(vec![reply]);
        let (request, body) = run(json!("NEW"));
        assert!(matches!(
            harness.client.actions().run(request).await,
            Err(Error::Accepted)
        ));
        assert_post(harness.request(1), "actions/run", &body);
        harness.assert_requests(2);
    }
}

#[tokio::test]
async fn structured_execution_failures_keep_attribution_and_provider_text_as_data() {
    for status in [200, 400, 503] {
        let harness = Harness::new(vec![response(
            status,
            json!({"error":{"name":"ProviderError","message":"provider message","attribution":{"origin":"upstream_api","network_calls_count":2,"last_call":{"status":400}}}}),
        )]);
        let (request, _) = run(json!(true));
        let error = harness.client.actions().run(request).await.unwrap_err();
        let Error::ExecutionFailed {
            error,
            status: actual,
        } = error
        else {
            panic!("expected execution attribution")
        };
        assert_eq!(actual, status);
        assert_eq!(error.attribution.unwrap().origin, ErrorOrigin::UpstreamApi);
        assert_eq!(error.name.as_deref(), Some("ProviderError"));
        assert_eq!(error.message.as_deref(), Some("provider message"));
        harness.assert_requests(2);
    }
}

#[tokio::test]
async fn non_execution_error_objects_keep_http_status_categories() {
    for status in [401, 404, 429] {
        let mut reply = response(status, json!({"error":{"message":"provider message"}}));
        if status == 429 {
            reply
                .headers
                .insert("retry-after", HeaderValue::from_static("7"));
        }
        let harness = Harness::new(vec![reply]);
        let (request, _) = run(json!(false));
        let error = harness.client.actions().run(request).await.unwrap_err();
        match (status, error) {
            (401, Error::Unauthorized { .. }) | (404, Error::NotFound { .. }) => {}
            (
                429,
                Error::RateLimited {
                    retry_after: Some(delay),
                    ..
                },
            ) => {
                assert_eq!(delay, Duration::from_secs(7));
            }
            (status, error) => panic!("wrong status mapping for {status}: {error:?}"),
        }
        harness.assert_requests(2);
    }
}

#[tokio::test]
async fn action_failures_never_retry_even_with_a_get_retry_policy() {
    for status in [429, 503] {
        let harness = Harness::script(
            config().with_retry_policy(RetryPolicy::new(
                NonZeroU32::new(3).unwrap(),
                Duration::from_secs(30),
                Duration::ZERO,
            )),
            vec![
                Ok(token("access-secret", 3600)),
                Ok(response(status, json!({"error":"failed"}))),
            ],
        );
        let (request, _) = run(json!(false));
        let result = harness.client.actions().run(request).await;
        assert!(matches!(
            (status, result),
            (429, Err(Error::RateLimited { .. }))
                | (
                    503,
                    Err(Error::Unavailable {
                        status: Some(503),
                        ..
                    })
                )
        ));
        harness.assert_requests(2);
    }
}
