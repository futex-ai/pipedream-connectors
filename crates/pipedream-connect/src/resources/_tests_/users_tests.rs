//! External-user list, account inventory, and deletion routes.

use pipedream_connect_types::{ExternalUserId, ListUserAccountsRequest, ListUsersRequest};
use serde_json::json;

use crate::resources::test_support::page;
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
    assert_eq!(
        harness.request(0).url.expose_secret(),
        "https://api.pipedream.com/v1/connect/proj_Test123/users?limit=100&q=user"
    );
    harness
        .client
        .users()
        .list_accounts(
            ExternalUserId::new("user_1").unwrap(),
            ListUserAccountsRequest::default(),
        )
        .await
        .unwrap();
    assert_eq!(
        harness.request(1).url.expose_secret(),
        "https://api.pipedream.com/v1/connect/proj_Test123/users/user_1/accounts?include_credentials=false"
    );
    harness
        .client
        .users()
        .delete(ExternalUserId::new("user_1").unwrap())
        .await
        .unwrap();
    assert_eq!(
        harness.request(2).url.expose_secret(),
        "https://api.pipedream.com/v1/connect/proj_Test123/users/user_1"
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
                ListUserAccountsRequest::default(),
            )
            .await;
        assert!(matches!(
            result,
            Err(crate::error::Error::NotFound { message: None })
        ));
        harness.assert_requests(1);
    }
}
