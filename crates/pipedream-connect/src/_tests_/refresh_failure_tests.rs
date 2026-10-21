//! Queued refresh callers share failures without a hold-off interval.

use std::future::{Future, poll_fn};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::task::Poll;
use std::time::{Duration, UNIX_EPOCH};

use http::HeaderValue;
use serde_json::json;
use tokio::sync::oneshot;
use unimock::{MockFn, Unimock, matching};

use crate::error::Error;
use crate::runtime::ClockMock;
use crate::token::{AccessTokenProvider, ClientCredentialsProvider};
use crate::transport::HttpTransportMock;

use super::hang_guard::guard;
use super::support::{config, response};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn queued_callers_share_a_failed_refresh_and_a_later_call_can_refresh() {
    let calls = Arc::new(AtomicUsize::new(0));
    let recorded = calls.clone();
    let (started_sender, started_receiver) = oneshot::channel();
    let started = Mutex::new(Some(started_sender));
    let (release_sender, release_receiver) = mpsc::channel();
    let release = Mutex::new(release_receiver);
    let transport = Arc::new(Unimock::new(
        HttpTransportMock::send
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                if recorded.fetch_add(1, Ordering::SeqCst) == 0 {
                    started.lock().unwrap().take().unwrap().send(()).unwrap();
                    release
                        .lock()
                        .unwrap()
                        .recv_timeout(Duration::from_secs(10))
                        .unwrap();
                }
                let mut reply = response(429, json!({"error":"throttled"}));
                reply
                    .headers
                    .insert("retry-after", HeaderValue::from_static("60"));
                Ok(reply)
            })),
    ));
    let clock = Arc::new(Unimock::new((
        ClockMock::monotonic
            .each_call(matching!())
            .returns(Duration::from_secs(1000)),
        ClockMock::now.each_call(matching!()).returns(UNIX_EPOCH),
    )));
    let provider = Arc::new(ClientCredentialsProvider::new(config(), transport, clock).unwrap());
    let first_provider = provider.clone();
    let first = tokio::spawn(async move { first_provider.access_token().await });
    guard(started_receiver).await.unwrap();
    let mut waiters: Vec<_> = (0..7).map(|_| Box::pin(provider.access_token())).collect();
    poll_fn(|context| {
        for waiter in &mut waiters {
            assert!(waiter.as_mut().poll(context).is_pending());
        }
        Poll::Ready(())
    })
    .await;
    release_sender.send(()).unwrap();
    let mut results = vec![guard(first).await.unwrap()];
    for waiter in waiters {
        results.push(guard(waiter).await);
    }
    for result in results {
        assert!(
            matches!(result, Err(Error::RateLimited { retry_after: Some(delay), .. }) if delay == Duration::from_secs(60))
        );
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(provider.access_token().await.is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}
