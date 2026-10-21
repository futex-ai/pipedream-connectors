//! Shared request recording through unimock transport, clock, and sleeper.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, UNIX_EPOCH};

use http::{HeaderMap, Method, StatusCode};
use pipedream_connect_types::{Environment, ProjectId, SecretString};
use serde_json::{Value, json};
use unimock::{MockFn, Unimock, matching};

use crate::client::PipedreamClient;
use crate::config::{Authentication, ClientConfig, ClientCredentials};
use crate::error::Result;
use crate::failure::check_limit;
use crate::request::{ApiPath, ApiRequest};
use crate::runtime::{ClockMock, SleeperMock};
use crate::transport::{HttpRequest, HttpResponse, HttpTransportMock};

pub(super) struct Harness {
    pub(super) client: PipedreamClient,
    pub(super) requests: Arc<Mutex<Vec<HttpRequest>>>,
    pub(super) now_millis: Arc<AtomicU64>,
    pub(super) wall_millis: Arc<AtomicU64>,
    pub(super) sleeps: Arc<Mutex<Vec<Duration>>>,
    replies: Arc<Mutex<VecDeque<Result<HttpResponse>>>>,
}

impl Harness {
    pub(super) fn new(responses: Vec<HttpResponse>) -> Self {
        let mut script = vec![Ok(token("access-secret", 3600))];
        script.extend(responses.into_iter().map(Ok));
        Self::script(config(), script)
    }

    pub(super) fn script(config: ClientConfig, responses: Vec<Result<HttpResponse>>) -> Self {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let replies = Arc::new(Mutex::new(VecDeque::from(responses)));
        let recorded = requests.clone();
        let scripted = replies.clone();
        let transport = Arc::new(Unimock::new(
            HttpTransportMock::send
                .each_call(matching!(_))
                .answers_arc(Arc::new(move |_, request| {
                    recorded.lock().unwrap().push(request);
                    scripted
                        .lock()
                        .unwrap()
                        .pop_front()
                        .expect("unexpected HTTP request")
                })),
        ));
        let now_millis = Arc::new(AtomicU64::new(1_000_000));
        let monotonic = now_millis.clone();
        let wall_millis = Arc::new(AtomicU64::new(1_000_000));
        let time = wall_millis.clone();
        let clock_needed = matches!(config.authentication, Authentication::ClientCredentials(_))
            || replies.lock().unwrap().iter().any(|reply| {
                reply.as_ref().is_ok_and(|reply| {
                    check_limit(reply, config.http_options().response_limit).is_ok()
                })
            });
        let wall = ClockMock::now
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                UNIX_EPOCH + Duration::from_millis(time.load(Ordering::SeqCst))
            }));
        let clock = if matches!(config.authentication, Authentication::ClientCredentials(_)) {
            Arc::new(Unimock::new((
                wall,
                ClockMock::monotonic
                    .each_call(matching!())
                    .answers_arc(Arc::new(move |_| {
                        Duration::from_millis(monotonic.load(Ordering::SeqCst))
                    })),
            )))
        } else if clock_needed {
            Arc::new(Unimock::new(wall))
        } else {
            Arc::new(Unimock::new(()))
        };
        let sleeps = Arc::new(Mutex::new(Vec::new()));
        let client =
            PipedreamClient::new(config, transport, clock, Arc::new(Unimock::new(()))).unwrap();
        Self {
            client,
            requests,
            now_millis,
            wall_millis,
            sleeps,
            replies,
        }
    }

    pub(super) fn record_sleeps(mut self) -> Self {
        let sleeps = self.sleeps.clone();
        self.client = self.client.with_sleeper(Arc::new(Unimock::new(
            SleeperMock::sleep
                .each_call(matching!(_))
                .answers_arc(Arc::new(move |_, duration| {
                    sleeps.lock().unwrap().push(duration)
                })),
        )));
        self
    }

    pub(super) async fn get(&self) -> Result<HttpResponse> {
        self.client.dispatcher().dispatch(get_request()).await
    }

    pub(super) fn request(&self, index: usize) -> HttpRequest {
        self.requests.lock().unwrap()[index].clone()
    }

    pub(super) fn assert_requests(&self, expected: usize) {
        assert_eq!(expected, self.requests.lock().unwrap().len());
        assert!(
            self.replies.lock().unwrap().is_empty(),
            "unused scripted responses"
        );
    }
}

pub(super) fn config() -> ClientConfig {
    ClientConfig::new(
        ProjectId::new("proj_Test123").unwrap(),
        Environment::Development,
        Authentication::ClientCredentials(
            ClientCredentials::new("client-id-secret", "client-secret").unwrap(),
        ),
    )
    .unwrap()
}

pub(super) fn static_config() -> ClientConfig {
    ClientConfig::new(
        ProjectId::new("proj_Test123").unwrap(),
        Environment::Development,
        Authentication::StaticBearer(SecretString::new("static-secret")),
    )
    .unwrap()
}

pub(super) fn get_request() -> ApiRequest {
    ApiRequest::new(Method::GET, ApiPath::project(&["apps"]))
}

pub(super) fn response(status: u16, value: Value) -> HttpResponse {
    HttpResponse {
        status: StatusCode::from_u16(status).unwrap(),
        headers: HeaderMap::new(),
        body: serde_json::to_vec(&value).unwrap(),
    }
}

pub(super) fn token(bearer: &str, expires_in: u32) -> HttpResponse {
    response(
        200,
        json!({"access_token":bearer,"token_type":"Bearer","expires_in":expires_in}),
    )
}
