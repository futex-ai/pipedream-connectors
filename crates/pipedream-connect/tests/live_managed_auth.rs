//! Credential-gated managed-auth and proxy smoke test.

#![cfg(feature = "reqwest")]

use std::env;

use http::{HeaderName, HeaderValue};
use pipedream_connect::PipedreamClient;
use pipedream_connect::config::ClientCredentials;
use pipedream_connect::resources::proxy::ProxyRequest;
use pipedream_connect_types::{
    AccountId, AccountRef, AppSlug, ConnectToken, CreateConnectTokenRequest, Environment,
    ExternalUserId, ListAccountsRequest, ProjectId, ValidateTokenRequest,
};

#[tokio::test]
async fn live_pipedream_managed_auth_and_proxy() {
    if env::var("PIPEDREAM_LIVE_TESTS").as_deref() != Ok("1") {
        eprintln!("skipping live Pipedream managed-auth smoke: set PIPEDREAM_LIVE_TESTS=1");
        return;
    }
    let names = [
        "PIPEDREAM_CLIENT_ID",
        "PIPEDREAM_CLIENT_SECRET",
        "PIPEDREAM_PROJECT_ID",
        "PIPEDREAM_EXTERNAL_USER_ID",
        "PIPEDREAM_CONNECTED_ACCOUNT_ID",
    ];
    let values = names.map(|name| env::var(name).ok().filter(|value| !value.trim().is_empty()));
    let missing: Vec<_> = names
        .iter()
        .zip(&values)
        .filter_map(|(name, value)| value.is_none().then_some(*name))
        .collect();
    if !missing.is_empty() {
        eprintln!(
            "skipping live Pipedream managed-auth smoke: missing {}",
            missing.join(", ")
        );
        return;
    }
    let [
        Some(client_id),
        Some(client_secret),
        Some(project_id),
        Some(external_user_id),
        Some(account_id),
    ] = values
    else {
        unreachable!("checked all managed-auth smoke variables")
    };
    let client = PipedreamClient::builder()
        .project_id(ProjectId::new(project_id).expect("valid live project ID"))
        .environment(Environment::Development)
        .client_credentials(
            ClientCredentials::new(client_id, client_secret).expect("valid live credentials"),
        )
        .build()
        .expect("live client configuration");
    let external_user_id = ExternalUserId::new(external_user_id).expect("valid external user ID");
    let account_id = AccountId::new(account_id).expect("valid connected account ID");
    let token_request = CreateConnectTokenRequest::new(external_user_id.clone())
        .with_expires_in(60)
        .expect("valid short-lived token request");
    let token = client
        .tokens()
        .create(token_request)
        .await
        .expect("create live Connect token");
    client
        .tokens()
        .validate(
            ConnectToken::new(token.token.expose_secret()).expect("valid returned token"),
            ValidateTokenRequest {
                app_id: AppSlug::new("google_calendar").expect("valid app slug"),
                account_id: Some(account_id.clone()),
                oauth_app_id: None,
            },
        )
        .await
        .expect("validate live Connect token");
    client
        .accounts()
        .list(ListAccountsRequest {
            external_user_id: Some(external_user_id.clone()),
            ..ListAccountsRequest::default()
        })
        .await
        .expect("list live accounts");
    let account = AccountRef::new(external_user_id, account_id);
    client
        .proxy()
        .get(ProxyRequest::get(
            account.clone(),
            "https://slack.com/api/auth.test",
        ))
        .await
        .expect("proxy GET smoke");
    client
        .proxy()
        .post(
            ProxyRequest::post(account, "https://slack.com/api/auth.test")
                .with_header(
                    HeaderName::from_static("content-type"),
                    HeaderValue::from_static("application/x-www-form-urlencoded"),
                )
                .with_body(Vec::<u8>::new()),
        )
        .await
        .expect("proxy POST smoke");
    eprintln!("managed-auth and proxy smoke completed; Connect token expires in 60 seconds");
}
