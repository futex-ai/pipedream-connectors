//! A shared hang guard for socket, channel, and concurrent task tests.

use std::future::Future;
use std::time::Duration;

pub(crate) async fn guard<F: Future>(future: F) -> F::Output {
    tokio::time::timeout(Duration::from_secs(10), future)
        .await
        .expect("test operation did not complete before the hang guard")
}
