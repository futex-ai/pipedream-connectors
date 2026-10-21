//! Configuration admission and exact transport defaults.

use std::sync::Arc;
use std::time::Duration;

use http::Method;
use pipedream_connect_types::{
    CreateConnectTokenRequest, Environment, ExternalUserId, PageRequest, ProjectId, SecretString,
};
use unimock::Unimock;

use crate::client::PipedreamClient;
use crate::config::{Authentication, ClientConfig, ClientCredentials};
use crate::error::{Error, InvalidRequest};
use crate::request::{ApiPath, ApiRequest};

use super::support::config;

#[test]
fn bare_https_origins_and_defaults_are_explicit() {
    let config = config();
    assert_eq!(
        config.http_options().connect_timeout,
        Duration::from_secs(10)
    );
    assert_eq!(
        config.http_options().request_timeout,
        Duration::from_secs(60)
    );
    assert_eq!(config.http_options().response_limit, 10 * 1024 * 1024);
    for valid in [
        "https://api.example.com",
        "https://api.example.com/",
        "https://api.example.com:8443",
        "HTTPS://api.example.com",
    ] {
        assert!(config.clone().with_origin(valid).is_ok(), "{valid}");
    }
    for invalid in [
        "http://api.example.com",
        "https://user:secret@api.example.com",
        "https://@api.example.com",
        "https://api.example.com/path",
        "https://api.example.com/.",
        "https://api.example.com/path/..",
        "https://api.example.com?secret=x",
        "https://api.example.com#secret",
        " https://api.example.com",
        "https://api.example.com\\sink",
        "https://api.exa\tmple.com",
        "invalid",
        "https://",
    ] {
        let error = config.clone().with_origin(invalid).unwrap_err();
        assert!(
            matches!(error, Error::InvalidRequest(InvalidRequest::Origin)),
            "{invalid}"
        );
        assert!(!format!("{error:?} {error}").contains("secret"));
    }
}

#[test]
fn credentials_and_header_values_are_validated_and_redacted() {
    for invalid in ["", " secret", "secret\n", "secret\0value"] {
        assert!(ClientCredentials::new("client", invalid).is_err());
        assert!(ClientCredentials::new(invalid, "secret").is_err());
        assert!(
            ClientConfig::new(
                ProjectId::new("proj_Test123").unwrap(),
                Environment::Development,
                Authentication::StaticBearer(SecretString::new(invalid))
            )
            .is_err()
        );
    }
    let config = config()
        .with_rate_limit_token(SecretString::new("rate-secret"))
        .unwrap();
    let diagnostic = format!("{config:?}");
    for secret in ["client-id-secret", "client-secret", "rate-secret"] {
        assert!(!diagnostic.contains(secret));
    }
    assert!(
        config
            .with_rate_limit_token(SecretString::new("bad\r\nvalue"))
            .is_err()
    );
    assert!(
        ClientConfig::new(
            ProjectId::new("proj_Test123").unwrap(),
            Environment::Other("future".into()),
            Authentication::StaticBearer(SecretString::new("bearer"))
        )
        .is_err()
    );
}

#[tokio::test]
async fn public_model_fields_are_validated_before_any_token_call() {
    let client = PipedreamClient::new(
        config(),
        Arc::new(Unimock::new(())),
        Arc::new(Unimock::new(())),
        Arc::new(Unimock::new(())),
    )
    .unwrap();
    let request =
        ApiRequest::new(Method::GET, ApiPath::project(&["apps"])).with_page(&PageRequest {
            limit: 0,
            ..PageRequest::default()
        });
    assert!(matches!(request, Err(Error::InvalidRequest(_))));
    let mut model = CreateConnectTokenRequest::new(ExternalUserId::new("user").unwrap());
    model.expires_in = 0;
    let request = ApiRequest::new(Method::POST, ApiPath::project(&["tokens"])).with_json(&model);
    assert!(matches!(request, Err(Error::InvalidRequest(_))));
    assert!(!format!("{client:?}").contains("client-secret"));
}

#[cfg(feature = "reqwest")]
#[test]
fn builder_requires_project_environment_and_authentication() {
    for builder in [
        PipedreamClient::builder(),
        PipedreamClient::builder().project_id(ProjectId::new("proj_Test123").unwrap()),
        PipedreamClient::builder()
            .project_id(ProjectId::new("proj_Test123").unwrap())
            .environment(Environment::Development),
    ] {
        assert!(matches!(
            builder.build(),
            Err(Error::InvalidRequest(
                InvalidRequest::MissingConfiguration { .. }
            ))
        ));
    }
    let builder = PipedreamClient::builder().origin("https://user:secret@invalid.example");
    assert!(!format!("{builder:?}").contains("secret@"));
}
