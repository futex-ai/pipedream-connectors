//! Account list, credential filters, identity checks, and cleanup routes.

use pipedream_connect_types::{
    AccountId, AppSlug, CreateAccountQuery, CreateAccountRequest, ExternalUserId,
    GetAccountRequest, ListAccountsRequest,
};
use serde_json::json;

use crate::error::Error;
use crate::resources::test_support::{assert_get, assert_wire, page};
use crate::tests::support::{Harness, response, static_config};

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
            oauth_app_id: Some(pipedream_connect_types::OauthAppId::new("oa_App1").unwrap()),
            page: pipedream_connect_types::PageRequest {
                after: Some(pipedream_connect_types::Cursor::new("after_1").unwrap()),
                before: None,
                limit: 25,
            },
            include_credentials: Some(true),
        })
        .await
        .unwrap();
    assert_get(
        harness.request(1),
        "https://api.pipedream.com/v1/connect/proj_Test123/accounts?after=after_1&app=google_calendar&external_user_id=user_1&include_credentials=true&limit=25&oauth_app_id=oa_App1",
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
            CreateAccountQuery {
                external_user_id: Some(ExternalUserId::new("user_1").unwrap()),
                oauth_app_id: None,
            },
        )
        .await
        .unwrap();
    let create_request = harness.request(4);
    assert_eq!(
        create_request.url.expose_secret(),
        "https://api.pipedream.com/v1/connect/proj_Test123/accounts?external_user_id=user_1"
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(create_request.body.as_ref().unwrap()).unwrap(),
        json!({
            "app_slug":"google_calendar",
            "cfmap_json":"{}",
            "connect_token":"ctok_0123456789abcdef0123456789abcdef"
        })
    );
    harness.client.accounts().delete(account_id).await.unwrap();
    assert_eq!(
        harness.request(5).url.expose_secret(),
        "https://api.pipedream.com/v1/connect/proj_Test123/accounts/apn_Account1"
    );
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
async fn account_list_preserves_before_and_oauth_app_queries() {
    let harness = Harness::new(vec![response(200, page(account()))]);
    harness
        .client
        .accounts()
        .list(ListAccountsRequest {
            page: pipedream_connect_types::PageRequest {
                before: Some(pipedream_connect_types::Cursor::new("before_1").unwrap()),
                ..pipedream_connect_types::PageRequest::default()
            },
            oauth_app_id: Some(pipedream_connect_types::OauthAppId::new("oa_App1").unwrap()),
            ..ListAccountsRequest::default()
        })
        .await
        .unwrap();
    assert_eq!(
        harness.request(1).url.expose_secret(),
        "https://api.pipedream.com/v1/connect/proj_Test123/accounts?before=before_1&include_credentials=false&limit=100&oauth_app_id=oa_App1"
    );
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
    assert_wire(
        &harness.request(2),
        http::Method::DELETE,
        "https://api.pipedream.com/v1/connect/proj_Test123/accounts/apn_Account1?external_user_id=user_1",
        &[
            ("authorization", "Bearer access-secret"),
            ("x-pd-environment", "development"),
            ("accept", "application/json"),
        ],
        None,
    );
    harness.assert_requests(3);
}

#[tokio::test]
async fn owned_account_mismatches_and_absent_external_ids_are_not_found() {
    for returned in [
        json!({"id":"apn_Account1","external_id":"other_user"}),
        json!({"id":"apn_Account1"}),
    ] {
        let harness = Harness::script(static_config(), vec![Ok(response(200, returned))]);
        let result = harness
            .client
            .accounts()
            .get_owned(
                pipedream_connect_types::AccountRef::new(
                    ExternalUserId::new("user_1").unwrap(),
                    AccountId::new("apn_Account1").unwrap(),
                ),
                GetAccountRequest::default(),
            )
            .await;
        assert!(matches!(result, Err(Error::NotFound { message: None })));
        harness.assert_requests(1);
    }
}

#[tokio::test]
async fn filtered_account_list_ownership_mismatches_are_not_found() {
    for account in [
        json!({"id":"apn_Account1","external_id":"other_user"}),
        json!({"id":"apn_Account1"}),
    ] {
        let harness = Harness::script(
            static_config(),
            vec![Ok(response(
                200,
                json!({"data":[account],"page_info":{"count":1}}),
            ))],
        );
        let result = harness
            .client
            .accounts()
            .list(ListAccountsRequest {
                external_user_id: Some(ExternalUserId::new("user_1").unwrap()),
                ..ListAccountsRequest::default()
            })
            .await;
        assert!(matches!(result, Err(Error::NotFound { message: None })));
        harness.assert_requests(1);
    }
}
