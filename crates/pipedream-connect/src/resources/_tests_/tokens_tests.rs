//! Managed Connect token request validation and wire contracts.

use pipedream_connect_types::{
    AccountId, AppSlug, ConnectToken, CreateConnectTokenRequest, ExternalUserId, OauthAppId,
    ValidateTokenRequest,
};
use serde_json::json;

use crate::error::Error;
use crate::resources::test_support::assert_wire;
use crate::tests::support::{Harness, config, response, static_config};

#[tokio::test]
async fn create_validates_urls_before_authentication_and_sends_the_exact_body() {
    let request = CreateConnectTokenRequest::new(ExternalUserId::new("user_1").unwrap())
        .with_success_redirect_uri("https://example.com/success")
        .unwrap()
        .with_error_redirect_uri("https://example.com/error")
        .unwrap()
        .with_webhook_uri("https://example.com/webhook")
        .unwrap();
    let mut request = request;
    request.allowed_origins = vec!["https://app.example.com".into()];
    request.scope = Some("connect:accounts:read".into());
    request.allow_progressive_scopes = Some(true);
    let harness = Harness::script(
        static_config(),
        vec![Ok(response(
            200,
            json!({
                "connect_link_url":"https://pipedream.com/connect?token=ctok_0123456789abcdef0123456789abcdef",
                "expires_at":"2026-10-21T12:00:00Z",
                "token":"ctok_0123456789abcdef0123456789abcdef"
            }),
        ))],
    );
    let result = harness.client.tokens().create(request).await.unwrap();
    assert_eq!(
        result.token.expose_secret(),
        "ctok_0123456789abcdef0123456789abcdef"
    );
    assert!(!format!("{result:?}").contains("ctok_0123456789abcdef0123456789abcdef"));
    assert_wire(
        &harness.request(0),
        http::Method::POST,
        "https://api.pipedream.com/v1/connect/proj_Test123/tokens",
        &[
            ("authorization", "Bearer static-secret"),
            ("x-pd-environment", "development"),
            ("accept", "application/json"),
            ("content-type", "application/json"),
        ],
        Some(&json!({
            "external_user_id":"user_1",
            "project_id":"proj_Test123",
            "expires_in":14400,
            "success_redirect_uri":"https://example.com/success",
            "error_redirect_uri":"https://example.com/error",
            "webhook_uri":"https://example.com/webhook"
            ,"allowed_origins":["https://app.example.com"]
            ,"scope":"connect:accounts:read"
            ,"allow_progressive_scopes":true
        })),
    );
    harness.assert_requests(1);

    let invalid = CreateConnectTokenRequest::new(ExternalUserId::new("user_1").unwrap());
    let invalid = invalid
        .with_success_redirect_uri("http://example.com/nope")
        .unwrap_err();
    assert!(matches!(
        invalid,
        pipedream_connect_types::Error::InvalidHttpsUrl { .. }
    ));
    let hash_route = CreateConnectTokenRequest::new(ExternalUserId::new("user_1").unwrap())
        .with_success_redirect_uri("https://example.com/#/done")
        .unwrap();
    assert_eq!(
        hash_route.success_redirect_uri.as_deref(),
        Some("https://example.com/#/done")
    );
    let harness = Harness::script(config(), vec![]);
    let mut request = CreateConnectTokenRequest::new(ExternalUserId::new("user_1").unwrap());
    request.expires_in = 0;
    let result = harness.client.tokens().create(request).await;
    assert!(matches!(result, Err(Error::InvalidRequest(_))));
    harness.assert_requests(0);

    let harness = Harness::script(config(), vec![]);
    let mut request = CreateConnectTokenRequest::new(ExternalUserId::new("user_1").unwrap());
    request.expires_in = 14_401;
    let result = harness.client.tokens().create(request).await;
    assert!(matches!(result, Err(Error::InvalidRequest(_))));
    harness.assert_requests(0);

    let harness = Harness::script(config(), vec![]);
    let mut request = CreateConnectTokenRequest::new(ExternalUserId::new("user_1").unwrap());
    request.success_redirect_uri = Some("http://example.com/unsafe".into());
    let result = harness.client.tokens().create(request).await;
    assert!(matches!(result, Err(Error::InvalidRequest(_))));
    harness.assert_requests(0);
}

#[tokio::test]
async fn validate_returns_the_complete_response_shape() {
    let harness = Harness::script(
        static_config(),
        vec![Ok(response(
            200,
            json!({
                "success": true,
                "error": null,
                "project_id": "proj_Test123",
                "project_environment": "development",
                "project_app_name": "Demo",
                "future": {"preserve": true}
            }),
        ))],
    );
    let result = harness
        .client
        .tokens()
        .validate(
            ConnectToken::new("ctok_0123456789abcdef0123456789abcdef").unwrap(),
            ValidateTokenRequest {
                app_id: AppSlug::new("google_calendar").unwrap(),
                account_id: None,
                oauth_app_id: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(result.project_id.as_ref().unwrap().as_str(), "proj_Test123");
    assert_eq!(result.extra["future"], json!({"preserve": true}));
    harness.assert_requests(1);
}

#[tokio::test]
async fn validate_encodes_the_connect_token_and_all_optional_filters() {
    let harness = Harness::script(
        static_config(),
        vec![Ok(response(200, json!({"success":true,"error":null})))],
    );
    harness
        .client
        .tokens()
        .validate(
            ConnectToken::new("ctok_0123456789abcdef0123456789abcdef").unwrap(),
            ValidateTokenRequest {
                app_id: AppSlug::new("google_calendar").unwrap(),
                account_id: Some(AccountId::new("apn_Account1").unwrap()),
                oauth_app_id: Some(OauthAppId::new("oa_App1").unwrap()),
            },
        )
        .await
        .unwrap();
    assert_wire(
        &harness.request(0),
        http::Method::GET,
        "https://api.pipedream.com/v1/connect/tokens/ctok_0123456789abcdef0123456789abcdef/validate?account_id=apn_Account1&app_id=google_calendar&oauth_app_id=oa_App1",
        &[
            ("authorization", "Bearer static-secret"),
            ("x-pd-environment", "development"),
            ("accept", "application/json"),
        ],
        None,
    );
    harness.assert_requests(1);
}
