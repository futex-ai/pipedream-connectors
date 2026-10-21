//! Concrete request construction failures and silent-server timeout mapping.

#![cfg(feature = "reqwest")]

#[path = "../src/_tests_/hang_guard.rs"]
mod hang_guard;

use std::time::Duration;

use http::{HeaderMap, Method};
use pipedream_connect::config::HttpOptions;
use pipedream_connect::error::Error;
use pipedream_connect::reqwest_transport::ReqwestTransport;
use pipedream_connect::transport::{HttpRequest, HttpTransport};
use pipedream_connect_types::SecretString;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;

use hang_guard::guard;

fn request(url: String) -> HttpRequest {
    HttpRequest {
        method: Method::GET,
        url: SecretString::new(url),
        headers: HeaderMap::new(),
        body: None,
        response_limit: 1024,
        timeout: Duration::from_millis(100),
    }
}

#[tokio::test]
async fn a_builder_error_is_internal_because_nothing_was_sent() {
    let transport = ReqwestTransport::new(&HttpOptions::default()).unwrap();
    let error = transport
        .send(request("invalid secret URL".into()))
        .await
        .unwrap_err();
    assert!(matches!(error, Error::Internal(_)));
    assert!(!format!("{error:?} {error}").contains("secret"));
}

#[tokio::test]
async fn a_silent_server_returns_unavailable_when_the_request_times_out() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/secret", listener.local_addr().unwrap());
    let (release_sender, release_receiver) = tokio::sync::oneshot::channel();
    let server = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut bytes = [0; 1024];
        assert_ne!(socket.read(&mut bytes).await.unwrap(), 0);
        guard(release_receiver).await.unwrap();
    });
    let transport = ReqwestTransport::new(&HttpOptions::default()).unwrap();
    let error = guard(transport.send(request(url))).await.unwrap_err();
    release_sender.send(()).unwrap();
    guard(server).await.unwrap();
    assert!(matches!(error, Error::Unavailable { status: None }));
    assert!(!format!("{error:?} {error}").contains("secret"));
}
