//! Local socket smoke tests for the optional concrete HTTP transport.

#![cfg(feature = "reqwest")]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use http::{HeaderMap, HeaderValue, Method};
use pipedream_connect::PipedreamClient;
use pipedream_connect::config::{ClientCredentials, HttpOptions};
use pipedream_connect::error::Error;
use pipedream_connect::reqwest_transport::ReqwestTransport;
use pipedream_connect::transport::{HttpRequest, HttpTransport};
use pipedream_connect_types::{Environment, ProjectId, SecretString};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

fn transport() -> Arc<dyn HttpTransport + Send + Sync> {
    Arc::new(ReqwestTransport::new(&HttpOptions::default()).unwrap())
}

#[tokio::test]
async fn exact_request_headers_and_body_and_repeated_response_headers_survive() {
    let (url, received) = server(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nX-Meta: first\r\nX-Meta: second\r\nX-Binary: \xff\r\nConnection: close\r\n\r\n{}".to_vec()).await;
    let mut outgoing = request(url, 1024);
    outgoing.method = Method::POST;
    outgoing
        .headers
        .insert("content-type", HeaderValue::from_static("application/json"));
    outgoing
        .headers
        .insert("accept", HeaderValue::from_static("application/json"));
    outgoing.headers.insert(
        "user-agent",
        HeaderValue::from_static(concat!("pipedream-connect/", env!("CARGO_PKG_VERSION"))),
    );
    outgoing
        .headers
        .insert("x-pd-rate-limit", HeaderValue::from_static("rate-secret"));
    outgoing.body = Some(b"{\"input\":true}".to_vec());
    let reply = transport().send(outgoing).await.unwrap();
    assert_eq!(reply.status, 200);
    assert_eq!(reply.body, b"{}");
    assert_eq!(
        reply.headers.get_all("x-meta").iter().collect::<Vec<_>>(),
        ["first", "second"]
    );
    assert_eq!(reply.headers["x-binary"].as_bytes(), &[0xff]);
    let wire = String::from_utf8(received.await.unwrap()).unwrap();
    assert!(wire.starts_with("POST /request HTTP/1.1\r\n"));
    for header in [
        "authorization: Bearer test-secret\r\n",
        "x-pd-environment: development\r\n",
        "content-type: application/json\r\n",
        "accept: application/json\r\n",
        "x-pd-rate-limit: rate-secret\r\n",
        concat!(
            "user-agent: pipedream-connect/",
            env!("CARGO_PKG_VERSION"),
            "\r\n"
        ),
    ] {
        assert!(wire.contains(header), "missing {header}");
    }
    assert!(wire.ends_with("\r\n\r\n{\"input\":true}"));
}

#[tokio::test]
async fn an_announced_oversize_body_is_rejected_without_reading_body_bytes() {
    let (url, received) =
        server(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\n".to_vec()).await;
    assert!(matches!(
        transport().send(request(url, 4)).await,
        Err(Error::ResponseTooLarge { limit: 4 })
    ));
    received.await.unwrap();
}

#[tokio::test]
async fn streaming_stops_at_the_cap_before_the_last_chunk_or_a_body_eof() {
    let (url, received) = server(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n3\r\nabc\r\n2\r\nde\r\n".to_vec()).await;
    assert!(matches!(
        transport().send(request(url, 4)).await,
        Err(Error::ResponseTooLarge { limit: 4 })
    ));
    received.await.unwrap();
}

#[tokio::test]
async fn exact_limits_and_empty_bodies_are_accepted() {
    for (limit, body) in [
        (0, Vec::new()),
        (4, b"abcd".to_vec()),
        (10 * 1024 * 1024, vec![b'x'; 10 * 1024 * 1024]),
    ] {
        let mut bytes = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .into_bytes();
        bytes.extend_from_slice(&body);
        let (url, received) = server(bytes).await;
        assert_eq!(
            transport().send(request(url, limit)).await.unwrap().body,
            body
        );
        received.await.unwrap();
    }
}

#[tokio::test]
async fn redirects_return_the_location_without_contacting_the_credential_sink() {
    let sink = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let location = format!("http://{}/credential-sink", sink.local_addr().unwrap());
    let accepted = Arc::new(AtomicUsize::new(0));
    let recorded = accepted.clone();
    let sink_task = tokio::spawn(async move {
        let (mut socket, _) = sink.accept().await.unwrap();
        recorded.fetch_add(1, Ordering::SeqCst);
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
            .await
            .unwrap();
    });
    let bytes = format!("HTTP/1.1 302 Found\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").into_bytes();
    let (url, received) = server(bytes).await;
    let reply = transport().send(request(url, 4)).await.unwrap();
    assert_eq!(reply.status, 302);
    assert_eq!(reply.headers["location"], location);
    assert_eq!(accepted.load(Ordering::SeqCst), 0);
    assert!(
        received
            .await
            .unwrap()
            .starts_with(b"GET /request HTTP/1.1\r\n")
    );
    sink_task.abort();
    assert!(sink_task.await.unwrap_err().is_cancelled());
}

#[tokio::test]
async fn incomplete_responses_return_a_sanitized_transport_failure() {
    let (url, received) =
        server(b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\nConnection: close\r\n\r\nsecret".to_vec())
            .await;
    let error = transport().send(request(url, 16)).await.unwrap_err();
    assert!(matches!(error, Error::Unavailable { status: None }));
    assert!(!format!("{error:?} {error}").contains("secret"));
    received.await.unwrap();
}

#[tokio::test]
async fn builder_constructs_the_default_client_without_provider_credentials() {
    let client = PipedreamClient::builder()
        .project_id(ProjectId::new("proj_Test123").unwrap())
        .environment(Environment::Development)
        .client_credentials(ClientCredentials::new("client-id", "client-secret").unwrap())
        .connect_timeout(Duration::from_secs(12))
        .request_timeout(Duration::from_secs(42))
        .response_limit(4096)
        .scope("read write")
        .build()
        .unwrap();
    assert_eq!(client.config().project_id().as_str(), "proj_Test123");
    assert_eq!(
        client.config().http_options().connect_timeout,
        Duration::from_secs(12)
    );
    assert_eq!(
        client.config().http_options().request_timeout,
        Duration::from_secs(42)
    );
    assert_eq!(client.config().http_options().response_limit, 4096);
}

async fn server(response: Vec<u8>) -> (String, JoinHandle<Vec<u8>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/request", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        loop {
            let mut bytes = [0; 1024];
            let read = socket.read(&mut bytes).await.unwrap();
            assert_ne!(read, 0);
            request.extend_from_slice(&bytes[..read]);
            assert!(request.len() < 16384);
            let Some(end) = request.windows(4).position(|value| value == b"\r\n\r\n") else {
                continue;
            };
            let headers = String::from_utf8_lossy(&request[..end]);
            let length = headers
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().unwrap())
                })
                .unwrap_or(0);
            if request.len() >= end + 4 + length {
                break;
            }
        }
        let _ = socket.write_all(&response).await;
        request
    });
    (url, task)
}

fn request(url: String, limit: usize) -> HttpRequest {
    let mut headers = HeaderMap::new();
    headers.insert(
        "authorization",
        HeaderValue::from_static("Bearer test-secret"),
    );
    headers.insert("x-pd-environment", HeaderValue::from_static("development"));
    HttpRequest {
        method: Method::GET,
        url: SecretString::new(url),
        headers,
        body: None,
        response_limit: limit,
        timeout: Duration::from_secs(60),
    }
}
