//! Connect token request limits, URI builders, and body tests.

use serde_json::json;

use crate::{
    CreateConnectTokenRequest, Error, ExternalUserId, ProjectId, Result, UrlField, ValidationField,
};

type SetUri = fn(CreateConnectTokenRequest, String) -> Result<CreateConnectTokenRequest>;

fn request() -> CreateConnectTokenRequest {
    CreateConnectTokenRequest::new(ExternalUserId::new("user_1").unwrap())
}

#[test]
fn expiry_boundaries_and_direct_values_are_validated() {
    for expiry in [0, 1, 14_400, 14_401] {
        let expected = (1..=14_400).contains(&expiry);
        assert_eq!(request().with_expires_in(expiry).is_ok(), expected);
        let mut direct = request();
        direct.expires_in = expiry;
        assert_eq!(direct.validate().is_ok(), expected);
    }
    assert!(matches!(
        request().with_expires_in(14_401),
        Err(Error::OutOfRange {
            field: ValidationField::ExpiresIn,
            ..
        })
    ));
}

#[test]
fn each_uri_builder_rejects_invalid_urls_without_leaking_values() {
    let builders: [(UrlField, SetUri); 3] = [
        (
            UrlField::SuccessRedirectUri,
            CreateConnectTokenRequest::with_success_redirect_uri,
        ),
        (
            UrlField::ErrorRedirectUri,
            CreateConnectTokenRequest::with_error_redirect_uri,
        ),
        (
            UrlField::WebhookUri,
            CreateConnectTokenRequest::with_webhook_uri,
        ),
    ];
    for (expected, set) in builders {
        assert!(set(request(), "https://example.com/return".into()).is_ok());
        for uri in [
            "http://example.com/return",
            "https://",
            "https:///",
            "not a URL",
            "https://[bad",
        ] {
            let error = set(request(), uri.into()).unwrap_err();
            assert!(matches!(error, Error::InvalidHttpsUrl {field} if field == expected));
            assert!(!error.to_string().contains(uri));
        }
    }
}

#[test]
fn encodes_plain_uris_and_the_documented_token_body() {
    let mut value = request()
        .with_project_id(ProjectId::new("proj_test").unwrap())
        .with_expires_in(1)
        .unwrap()
        .with_success_redirect_uri("https://example.com/success")
        .unwrap()
        .with_error_redirect_uri("https://example.com/error")
        .unwrap()
        .with_webhook_uri("https://example.com/webhook")
        .unwrap();
    value.allowed_origins = vec!["https://example.com".into()];
    value.scope = Some("connect:*".into());
    value.allow_progressive_scopes = Some(true);
    assert!(format!("{value:?}").contains("https://example.com/success"));
    assert_eq!(
        serde_json::to_value(value).unwrap(),
        json!({
            "project_id":"proj_test", "external_user_id":"user_1", "expires_in":1,
            "allowed_origins":["https://example.com"], "success_redirect_uri":"https://example.com/success",
            "error_redirect_uri":"https://example.com/error", "webhook_uri":"https://example.com/webhook",
            "scope":"connect:*", "allow_progressive_scopes":true
        })
    );
}

#[test]
fn deserialization_checks_expiry_boundaries_and_keeps_the_default() {
    for expiry in [0, 1, 14_400, 14_401] {
        let decoded = serde_json::from_value::<CreateConnectTokenRequest>(json!({
            "external_user_id": "user_1", "expires_in": expiry
        }));
        assert_eq!(decoded.is_ok(), (1..=14_400).contains(&expiry));
    }
    let decoded: CreateConnectTokenRequest =
        serde_json::from_value(json!({"external_user_id": "user_1"})).unwrap();
    assert_eq!(decoded, request());
}
