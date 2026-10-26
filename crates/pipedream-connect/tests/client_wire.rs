//! Exact token and Connect wire requests through client dispatch and reqwest.

#![cfg(feature = "reqwest")]
#![allow(clippy::disallowed_methods)]

mod _support;

use std::sync::Arc;

use async_trait::async_trait;
use http::Method;
use pipedream_connect::PipedreamClient;
use pipedream_connect::config::{Authentication, ClientConfig, ClientCredentials, HttpOptions};
use pipedream_connect::error::Result;
use pipedream_connect::request::{ApiPath, ApiRequest};
use pipedream_connect::reqwest_transport::ReqwestTransport;
use pipedream_connect::runtime::{SystemClock, TokioSleeper};
use pipedream_connect::transport::{HttpRequest, HttpResponse, HttpTransport};
use pipedream_connect_types::{
    CreateConnectTokenRequest, Environment, ExternalUserId, ProjectId, SecretString,
};

use _support::hang_guard::guard;
use _support::server::server;

/// Test-only routing keeps the public client origin restricted to HTTPS.
struct Loopback {
    oauth: String,
    connect: String,
    transport: Arc<dyn HttpTransport + Send + Sync>,
}

#[async_trait]
impl HttpTransport for Loopback {
    async fn send(&self, mut request: HttpRequest) -> Result<HttpResponse> {
        let path = request
            .url
            .expose_secret()
            .strip_prefix("https://api.pipedream.com")
            .unwrap();
        let local = if path == "/v1/oauth/token" {
            &self.oauth
        } else {
            &self.connect
        };
        let origin = local.strip_suffix("/request").unwrap();
        request.url = SecretString::new(format!("{origin}{path}"));
        self.transport.send(request).await
    }
}

fn headers(body: &[u8], url: &str, expected: Vec<String>, first_line: &str, expected_body: &[u8]) {
    let wire = String::from_utf8(body.to_vec()).unwrap();
    let (head, body) = wire.split_once("\r\n\r\n").unwrap();
    assert_eq!(body.as_bytes(), expected_body);
    let mut lines = head.split("\r\n");
    assert_eq!(lines.next(), Some(first_line));
    let mut actual: Vec<_> = lines.map(str::to_owned).collect();
    actual.sort();
    let mut expected = expected;
    expected.push(format!(
        "host: {}",
        url.strip_prefix("http://")
            .unwrap()
            .strip_suffix("/request")
            .unwrap()
    ));
    expected.push(format!("content-length: {}", expected_body.len()));
    expected.sort();
    assert_eq!(
        actual, expected,
        "unexpected, missing, or repeated wire headers"
    );
}

#[tokio::test]
async fn client_token_and_connect_requests_have_no_extra_headers() {
    let token_body = br#"{"access_token":"wire-bearer","token_type":"Bearer","expires_in":3600}"#;
    let token_response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        token_body.len(),
        String::from_utf8_lossy(token_body)
    );
    let (oauth_url, oauth_wire) = server(token_response.into_bytes()).await;
    let (connect_url, connect_wire) =
        server(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}".to_vec())
            .await;
    let transport = Arc::new(Loopback {
        oauth: oauth_url.clone(),
        connect: connect_url.clone(),
        transport: Arc::new(ReqwestTransport::new(&HttpOptions::default()).unwrap()),
    });
    let config = ClientConfig::new(
        ProjectId::new("proj_Wire123").unwrap(),
        Environment::Production,
        Authentication::ClientCredentials(
            ClientCredentials::new("wire-client", "wire-secret").unwrap(),
        ),
    )
    .unwrap()
    .with_scope("read write")
    .with_rate_limit_token(SecretString::new("wire-rate"))
    .unwrap();
    let client = PipedreamClient::new(
        config,
        transport,
        Arc::new(SystemClock::default()),
        Arc::new(TokioSleeper),
    )
    .unwrap();
    let model = CreateConnectTokenRequest::new(ExternalUserId::new("wire-user").unwrap());
    let request = ApiRequest::new(Method::POST, ApiPath::project(&["tokens"]))
        .with_json(&model)
        .unwrap();
    guard(client.dispatcher().dispatch(request)).await.unwrap();
    let common = vec![
        "accept: application/json".to_owned(),
        "content-type: application/json".to_owned(),
        concat!("user-agent: pipedream-connect/", env!("CARGO_PKG_VERSION")).to_owned(),
    ];
    headers(&guard(oauth_wire).await.unwrap(),&oauth_url,common.clone(),"POST /v1/oauth/token HTTP/1.1",
        br#"{"grant_type":"client_credentials","client_id":"wire-client","client_secret":"wire-secret","scope":"read write"}"#);
    let mut connect = common;
    connect.extend([
        "authorization: Bearer wire-bearer".into(),
        "x-pd-environment: production".into(),
        "x-pd-rate-limit: wire-rate".into(),
    ]);
    headers(
        &guard(connect_wire).await.unwrap(),
        &connect_url,
        connect,
        "POST /v1/connect/proj_Wire123/tokens HTTP/1.1",
        br#"{"external_user_id":"wire-user","expires_in":14400}"#,
    );
}
