//! External-user list, account inventory, and deletion routes.

use pipedream_connect_types::{AppSlug, ExternalUserId, ListUserAccountsRequest, ListUsersRequest};
use serde_json::json;

use crate::error::Error;
use crate::resources::test_support::{assert_wire, page};
use crate::tests::support::{Harness, response, static_config};

#[tokio::test]
async fn users_list_accounts_and_delete_use_tenant_scoped_paths() {
    let harness = Harness::script(
        static_config(),
        vec![
            Ok(response(
                200,
                page(json!({"id":"user_1","external_id":"user_1"})),
            )),
            Ok(response(
                200,
                json!([{"id":"apn_Account1","external_id":"user_1"}]),
            )),
            Ok(response(204, json!(null))),
        ],
    );
    harness
        .client
        .users()
        .list(ListUsersRequest {
            query: Some("user".into()),
            ..ListUsersRequest::default()
        })
        .await
        .unwrap();
    assert_wire(
        &harness.request(0),
        http::Method::GET,
        "https://api.pipedream.com/v1/connect/proj_Test123/users?limit=100&q=user",
        &[
            ("authorization", "Bearer static-secret"),
            ("x-pd-environment", "development"),
            ("accept", "application/json"),
            ("accept", "application/json"),
        ],
        None,
    );
    harness
        .client
        .users()
        .list_accounts(
            ExternalUserId::new("user_1").unwrap(),
            ListUserAccountsRequest {
                app: Some(AppSlug::new("google_calendar").unwrap()),
                ..ListUserAccountsRequest::default()
            },
        )
        .await
        .unwrap();
    assert_wire(
        &harness.request(1),
        http::Method::GET,
        "https://api.pipedream.com/v1/connect/proj_Test123/users/user_1/accounts?app=google_calendar&include_credentials=false",
        &[
            ("authorization", "Bearer static-secret"),
            ("x-pd-environment", "development"),
            ("accept", "application/json"),
        ],
        None,
    );
    harness
        .client
        .users()
        .delete(ExternalUserId::new("user_1").unwrap())
        .await
        .unwrap();
    assert_wire(
        &harness.request(2),
        http::Method::DELETE,
        "https://api.pipedream.com/v1/connect/proj_Test123/users/user_1",
        &[
            ("authorization", "Bearer static-secret"),
            ("x-pd-environment", "development"),
        ],
        None,
    );
    harness.assert_requests(3);
}

#[tokio::test]
async fn user_account_ownership_mismatches_are_not_found() {
    for account in [
        json!({"id":"apn_Account1","external_id":"other_user"}),
        json!({"id":"apn_Account1"}),
    ] {
        let harness = Harness::script(static_config(), vec![Ok(response(200, json!([account])))]);
        let result = harness
            .client
            .users()
            .list_accounts(
                ExternalUserId::new("user_1").unwrap(),
                ListUserAccountsRequest {
                    app: Some(pipedream_connect_types::AppSlug::new("google_calendar").unwrap()),
                    ..ListUserAccountsRequest::default()
                },
            )
            .await;
        assert!(matches!(result, Err(Error::NotFound { message: None })));
        harness.assert_requests(1);
    }
}
