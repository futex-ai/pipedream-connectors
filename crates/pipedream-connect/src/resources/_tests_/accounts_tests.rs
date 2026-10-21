//! Account list, credential filters, identity checks, and cleanup routes.

use pipedream_connect_types::{
    AccountId, AppSlug, CreateAccountRequest, ExternalUserId, GetAccountRequest,
    ListAccountsRequest,
};
use serde_json::json;

use crate::error::Error;
use crate::resources::test_support::{assert_get, assert_post, page};
use crate::tests::support::{Harness, response};

fn account() -> serde_json::Value {
    json!({
        "id":"apn_Account1",
        "external_id":"user_1",
        "credentials":{"oauth_access_token":"account-secret"},
        "app":{"name_slug":"google_calendar","name":"Google Calendar","img_src":"https://example.com/logo"}
    })
}

#[tokio::test]
async fn account_operations_encode_filters_and_keep_credentials_redacted() {
    let harness = Harness::new(vec![
        response(200, page(account())),
        response(200, account()),
        response(200, account()),
        response(200, account()),
        response(204, json!(null)),
        response(204, json!(null)),
    ]);
    let page = harness
        .client
        .accounts()
        .list(ListAccountsRequest {
            external_user_id: Some(ExternalUserId::new("user_1").unwrap()),
            app: Some(AppSlug::new("google_calendar").unwrap()),
            include_credentials: Some(true),
            ..ListAccountsRequest::default()
        })
        .await
        .unwrap();
    assert_get(
        harness.request(1),
        "https://api.pipedream.com/v1/connect/proj_Test123/accounts?app=google_calendar&external_user_id=user_1&include_credentials=true&limit=100",
    );
    assert!(!format!("{page:?}").contains("account-secret"));
    let account_id = AccountId::new("apn_Account1").unwrap();
    let result = harness
        .client
        .accounts()
        .get(account_id.clone(), GetAccountRequest::default())
        .await
        .unwrap();
    assert!(!format!("{result:?}").contains("account-secret"));
    assert_get(
        harness.request(2),
        "https://api.pipedream.com/v1/connect/proj_Test123/accounts/apn_Account1?include_credentials=false",
    );
    harness
        .client
        .accounts()
        .get(
            AccountId::new("apn_Account1").unwrap(),
            GetAccountRequest {
                include_credentials: Some(true),
            },
        )
        .await
        .unwrap();
    assert_get(
        harness.request(3),
        "https://api.pipedream.com/v1/connect/proj_Test123/accounts/apn_Account1?include_credentials=true",
    );
    harness
        .client
        .accounts()
        .create(
            serde_json::from_value::<CreateAccountRequest>(json!({
                "app_slug":"google_calendar",
                "cfmap_json":"{}",
                "connect_token":"ctok_0123456789abcdef0123456789abcdef"
            }))
            .unwrap(),
        )
        .await
        .unwrap();
    assert_post(
        harness.request(4),
        "accounts",
        &json!({
            "app_slug":"google_calendar",
            "cfmap_json":"{}",
            "connect_token":"ctok_0123456789abcdef0123456789abcdef"
        }),
    );
    harness.client.accounts().delete(account_id).await.unwrap();
    assert_eq!(harness.request(5).method, http::Method::DELETE);
    harness
        .client
        .accounts()
        .delete_by_app(AppSlug::new("google_calendar").unwrap())
        .await
        .unwrap();
    assert_eq!(
        harness.request(6).url.expose_secret(),
        "https://api.pipedream.com/v1/connect/proj_Test123/apps/google_calendar/accounts"
    );
    harness.assert_requests(7);
}

#[tokio::test]
async fn account_get_rejects_a_returned_identity_that_does_not_match() {
    let mut value = account();
    value["id"] = json!("apn_Other");
    let harness = Harness::new(vec![response(200, value)]);
    let result = harness
        .client
        .accounts()
        .get(
            AccountId::new("apn_Account1").unwrap(),
            GetAccountRequest::default(),
        )
        .await;
    assert!(matches!(result, Err(Error::Internal(_))));
    harness.assert_requests(2);
}

#[tokio::test]
async fn owned_account_cleanup_checks_the_external_user_before_delete() {
    let harness = Harness::new(vec![response(200, account()), response(204, json!(null))]);
    let reference = pipedream_connect_types::AccountRef::new(
        ExternalUserId::new("user_1").unwrap(),
        AccountId::new("apn_Account1").unwrap(),
    );
    harness
        .client
        .accounts()
        .delete_owned(reference)
        .await
        .unwrap();
    assert_eq!(
        harness.request(1).url.expose_secret(),
        "https://api.pipedream.com/v1/connect/proj_Test123/accounts/apn_Account1?include_credentials=false&external_user_id=user_1"
    );
    assert_eq!(harness.request(2).method, http::Method::DELETE);
    harness.assert_requests(3);
}
