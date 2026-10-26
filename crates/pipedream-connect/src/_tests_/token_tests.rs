//! Early refresh, bearer invalidation, and failed refresh admission.

use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, UNIX_EPOCH};

use pipedream_connect_types::SecretString;
use serde_json::json;
use unimock::{MockFn, Unimock, matching};

use crate::error::Error;
use crate::runtime::ClockMock;
use crate::token::{AccessTokenProvider, ClientCredentialsProvider, StaticTokenProvider};
use crate::transport::HttpTransportMock;

use super::support::{Harness, config, response, token};

#[tokio::test]
async fn refresh_uses_the_smaller_of_thirty_seconds_or_ten_percent() {
    for (lifetime, refresh_millis) in [(100, 90_000), (3600, 3_570_000), (1, 900)] {
        let harness = Harness::script(
            config(),
            vec![
                Ok(token("first", lifetime)),
                Ok(response(200, json!({}))),
                Ok(response(200, json!({}))),
                Ok(token("second", lifetime)),
                Ok(response(200, json!({}))),
            ],
        );
        harness.get().await.unwrap();
        harness
            .now_millis
            .store(1_000_000 + refresh_millis - 1, Ordering::SeqCst);
        harness.get().await.unwrap();
        assert_eq!(harness.request(2).headers["authorization"], "Bearer first");
        harness
            .now_millis
            .store(1_000_000 + refresh_millis, Ordering::SeqCst);
        harness.get().await.unwrap();
        assert_eq!(harness.request(4).headers["authorization"], "Bearer second");
        harness.assert_requests(5);
    }
}

#[tokio::test]
async fn unauthorized_invalidates_the_used_bearer_without_replaying() {
    let harness = Harness::script(
        config(),
        vec![
            Ok(token("old", 3600)),
            Ok(response(401, json!({"error":"rejected"}))),
            Ok(token("fresh", 3600)),
            Ok(response(200, json!({}))),
        ],
    );
    assert!(matches!(
        harness.get().await,
        Err(Error::Unauthorized { .. })
    ));
    assert_eq!(harness.requests.lock().unwrap().len(), 2);
    harness.get().await.unwrap();
    assert_eq!(harness.request(3).headers["authorization"], "Bearer fresh");
    harness.assert_requests(4);
}

#[tokio::test]
async fn invalidating_an_old_bearer_does_not_clear_a_newer_bearer() {
    let transport = Arc::new(Unimock::new((
        HttpTransportMock::send
            .next_call(matching!(_))
            .returns(Ok(token("first", 100))),
        HttpTransportMock::send
            .next_call(matching!(_))
            .returns(Ok(token("second", 100))),
    )));
    let time = Arc::new(std::sync::atomic::AtomicU64::new(1000));
    let clock_time = time.clone();
    let clock = Arc::new(Unimock::new((
        ClockMock::now.each_call(matching!()).returns(UNIX_EPOCH),
        ClockMock::monotonic
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                Duration::from_secs(clock_time.load(Ordering::SeqCst))
            })),
    )));
    let provider: Arc<dyn AccessTokenProvider + Send + Sync> =
        Arc::new(ClientCredentialsProvider::new(config(), transport, clock).unwrap());
    let old = provider.access_token().await.unwrap();
    time.store(1090, Ordering::SeqCst);
    assert_eq!(
        provider.access_token().await.unwrap().expose_secret(),
        "second"
    );
    provider.invalidate(&old).await;
    assert_eq!(
        provider.access_token().await.unwrap().expose_secret(),
        "second"
    );
}

#[tokio::test]
async fn failed_refresh_is_not_cached_and_invalid_responses_never_expose_values() {
    let harness = Harness::script(
        config(),
        vec![
            Err(Error::Unavailable {
                status: None,
                retry_after: None,
            }),
            Ok(token("fresh", 3600)),
            Ok(response(200, json!({}))),
        ],
    );
    assert!(matches!(
        harness.get().await,
        Err(Error::Unavailable {
            status: None,
            retry_after: None
        })
    ));
    harness.get().await.unwrap();
    harness.assert_requests(3);
    for reply in [
        token("", 3600),
        token("bad\r\nsecret", 3600),
        token("valid", 0),
        response(
            200,
            json!({"access_token":"secret", "token_type":"unrecognized", "expires_in":3600}),
        ),
        response(
            200,
            json!({"access_token":"secret", "token_type":"Bearer", "expires_in":"secret"}),
        ),
    ] {
        let harness = Harness::script(config(), vec![Ok(reply)]);
        let error = harness.get().await.unwrap_err();
        assert!(matches!(error, Error::Internal(_)));
        assert!(!format!("{error:?} {error}").contains("secret"));
        harness.assert_requests(1);
    }
}

#[tokio::test]
async fn a_token_that_expires_during_acquisition_is_never_dispatched() {
    let clock = Arc::new(Unimock::new((
        ClockMock::monotonic
            .next_call(matching!())
            .returns(Duration::from_secs(1000)),
        ClockMock::now
            .next_call(matching!())
            .returns(UNIX_EPOCH + Duration::from_secs(1091)),
        ClockMock::monotonic
            .next_call(matching!())
            .returns(Duration::from_secs(1091)),
    )));
    let transport = Arc::new(Unimock::new(
        HttpTransportMock::send
            .next_call(matching!(_))
            .returns(Ok(token("expired", 100))),
    ));
    let provider: Arc<dyn AccessTokenProvider + Send + Sync> =
        Arc::new(ClientCredentialsProvider::new(config(), transport, clock).unwrap());
    assert!(matches!(
        provider.access_token().await,
        Err(Error::Unavailable {
            status: None,
            retry_after: None
        })
    ));
}

#[tokio::test]
async fn static_bearers_have_no_oauth_cache_or_refresh_side_effect() {
    let provider: Arc<dyn AccessTokenProvider + Send + Sync> =
        Arc::new(StaticTokenProvider::new(SecretString::new("static-secret")).unwrap());
    let bearer = provider.access_token().await.unwrap();
    provider.invalidate(&bearer).await;
    assert_eq!(provider.access_token().await.unwrap(), bearer);
    assert!(!format!("{:?}", StaticTokenProvider::new(bearer).unwrap()).contains("static-secret"));
}
