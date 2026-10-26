//! Custom OAuth app request and redaction tests.

use pipedream_connect_types::{
    CreateOauthAppRequest, ListOauthAppsRequest, OauthAppId, SecretString, UpdateOauthAppRequest,
};
use serde_json::json;

use crate::error::Error;
use crate::resources::test_support::assert_wire;
use crate::tests::support::{Harness, response, static_config};

fn app() -> serde_json::Value {
    json!({"id":"oa_Test123","client_id":"client","owner_id":"proj_Test123"})
}

#[tokio::test]
async fn oauth_app_operations_encode_routes_and_keep_secret_debug_redacted() {
    let harness = Harness::script(
        static_config(),
        vec![
            Ok(response(200, json!({"data":[],"page_info":{"count":0}}))),
            Ok(response(201, app())),
            Ok(response(200, app())),
            Ok(response(
                200,
                json!({"id":"oa_Other","client_id":"client","owner_id":"proj_Test123"}),
            )),
            Ok(response(204, json!(null))),
        ],
    );
    harness
        .client
        .oauth_apps()
        .list(ListOauthAppsRequest::default())
        .await
        .unwrap();
    harness
        .client
        .oauth_apps()
        .create(CreateOauthAppRequest {
            app: "github".into(),
            client_id: "client".into(),
            client_secret: SecretString::new("oauth-secret"),
            name: None,
            description: None,
            scopes: None,
            additional_scopes: None,
        })
        .await
        .unwrap();
    assert!(!format!("{:?}", harness.request(1)).contains("oauth-secret"));
    let id = OauthAppId::new("oa_Test123").unwrap();
    harness.client.oauth_apps().get(id.clone()).await.unwrap();
    harness
        .client
        .oauth_apps()
        .update(
            id.clone(),
            UpdateOauthAppRequest {
                client_secret: Some(SecretString::new("new-secret")),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert_wire(
        &harness.request(2),
        http::Method::GET,
        "https://api.pipedream.com/v1/connect/proj_Test123/oauth_apps/oa_Test123",
        &[
            ("authorization", "Bearer static-secret"),
            ("x-pd-environment", "development"),
            ("accept", "application/json"),
        ],
        None,
    );
    harness.client.oauth_apps().delete(id).await.unwrap();
    harness.assert_requests(5);
}

#[tokio::test]
async fn oauth_app_get_rejects_a_response_for_another_app() {
    let harness = Harness::script(
        static_config(),
        vec![Ok(response(
            200,
            json!({"id":"oa_Other","client_id":"client","owner_id":"proj_Test123"}),
        ))],
    );
    let result = harness
        .client
        .oauth_apps()
        .get(OauthAppId::new("oa_Test123").unwrap())
        .await;

    assert!(matches!(result, Err(Error::Internal(_))));
    assert_wire(
        &harness.request(0),
        http::Method::GET,
        "https://api.pipedream.com/v1/connect/proj_Test123/oauth_apps/oa_Test123",
        &[
            ("authorization", "Bearer static-secret"),
            ("x-pd-environment", "development"),
            ("accept", "application/json"),
        ],
        None,
    );
    harness.assert_requests(1);
}
