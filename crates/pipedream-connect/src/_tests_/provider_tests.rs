//! Explicit token-provider injection and safe request serialization failures.

use std::sync::Arc;
use std::time::UNIX_EPOCH;

use http::Method;
use pipedream_connect_types::SecretString;
use serde::{Serialize, Serializer};
use unimock::{MockFn, Unimock, matching};

use crate::client::PipedreamClient;
use crate::error::Error;
use crate::request::{ApiPath, ApiRequest};
use crate::runtime::ClockMock;
use crate::token::AccessTokenProviderMock;
use crate::transport::HttpTransportMock;
use crate::validation::Validate;

use super::support::{response, static_config};

#[tokio::test]
async fn a_401_invalidates_the_exact_injected_bearer_without_replay() {
    let tokens = Arc::new(Unimock::new((
        AccessTokenProviderMock::access_token
            .next_call(matching!())
            .returns(Ok(SecretString::new("injected-secret"))),
        AccessTokenProviderMock::invalidate
            .next_call(matching!((bearer) if bearer.expose_secret() == "injected-secret"))
            .returns(()),
    )));
    let transport = Arc::new(Unimock::new(HttpTransportMock::send
        .next_call(matching!((request) if request.headers["authorization"] == "Bearer injected-secret"))
        .returns(Ok(response(401, serde_json::json!({}))))));
    let clock = Arc::new(Unimock::new(
        ClockMock::now.next_call(matching!()).returns(UNIX_EPOCH),
    ));
    let client = PipedreamClient::new(
        static_config(),
        transport,
        clock,
        Arc::new(Unimock::new(())),
    )
    .unwrap()
    .with_token_provider(tokens);
    let request = ApiRequest::new(Method::POST, ApiPath::project(&["actions", "run"]));
    assert!(matches!(
        client.dispatcher().dispatch(request).await,
        Err(Error::Unauthorized { .. })
    ));
}

struct SensitiveSerialization;

impl Serialize for SensitiveSerialization {
    fn serialize<S: Serializer>(&self, _serializer: S) -> std::result::Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom("sensitive-secret"))
    }
}

#[test]
fn serialization_failures_retain_no_serializer_error_text() {
    let request = ApiRequest::new(Method::POST, ApiPath::project(&["tokens"]))
        .with_json(&SensitiveSerialization);
    let error = request.unwrap_err();
    assert!(matches!(error, Error::Internal(_)));
    assert!(!format!("{error:?} {error}").contains("sensitive-secret"));
}

impl Validate for SensitiveSerialization {
    fn validate(&self) -> crate::error::Result<()> {
        Ok(())
    }
}
