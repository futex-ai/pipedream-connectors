//! Concurrent callers share an explicitly blocked refresh without timing checks.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, UNIX_EPOCH};

use tokio::sync::{Notify, oneshot};
use unimock::{MockFn, Unimock, matching};

use crate::runtime::ClockMock;
use crate::token::{AccessTokenProvider, ClientCredentialsProvider};
use crate::transport::HttpTransportMock;

use super::support::{config, token};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_callers_wait_for_one_in_flight_refresh() {
    let calls = Arc::new(AtomicUsize::new(0));
    let recorded = calls.clone();
    let (started_sender, started_receiver) = oneshot::channel();
    let started = Mutex::new(Some(started_sender));
    let (release_sender, release_receiver) = mpsc::channel();
    let release = Mutex::new(release_receiver);
    let transport = Arc::new(Unimock::new(
        HttpTransportMock::send
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, request| {
                assert!(request.url.expose_secret().ends_with("/v1/oauth/token"));
                if recorded.fetch_add(1, Ordering::SeqCst) == 0 {
                    started.lock().unwrap().take().unwrap().send(()).unwrap();
                    release.lock().unwrap().recv().unwrap();
                }
                Ok(token("shared", 3600))
            })),
    ));
    let clock = Arc::new(Unimock::new(
        ClockMock::now
            .each_call(matching!())
            .returns(UNIX_EPOCH + Duration::from_secs(1000)),
    ));
    let provider: Arc<dyn AccessTokenProvider + Send + Sync> =
        Arc::new(ClientCredentialsProvider::new(config(), transport, clock).unwrap());
    let first_provider = provider.clone();
    let first = tokio::spawn(async move { first_provider.access_token().await.unwrap() });
    started_receiver.await.unwrap();
    let arrivals = Arc::new(AtomicUsize::new(0));
    let arrived = Arc::new(Notify::new());
    let mut others = Vec::new();
    for _ in 0..31 {
        let provider = provider.clone();
        let arrivals = arrivals.clone();
        let arrived = arrived.clone();
        others.push(tokio::spawn(async move {
            arrivals.fetch_add(1, Ordering::SeqCst);
            arrived.notify_one();
            provider.access_token().await.unwrap()
        }));
    }
    while arrivals.load(Ordering::SeqCst) < 31 {
        arrived.notified().await;
    }
    let requests_while_pending = calls.load(Ordering::SeqCst);
    let completed_while_pending = others.iter().filter(|task| task.is_finished()).count();
    release_sender.send(()).unwrap();
    assert_eq!(first.await.unwrap().expose_secret(), "shared");
    for task in others {
        assert_eq!(task.await.unwrap().expose_secret(), "shared");
    }
    assert_eq!(requests_while_pending, 1);
    assert_eq!(completed_while_pending, 0);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
