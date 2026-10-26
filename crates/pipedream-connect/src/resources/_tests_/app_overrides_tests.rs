//! App override request and identity tests.

use pipedream_connect_types::{
    AppOverrideId, CreateAppOverrideRequest, ListAppOverridesRequest, UpdateAppOverrideRequest,
};
use serde_json::json;

use crate::error::Error;
use crate::resources::test_support::assert_wire;
use crate::tests::support::{Harness, response, static_config};

fn override_value(id: &str) -> serde_json::Value {
    json!({"id":id,"name":"Override","app":{"name_slug":"github","name":"GitHub","img_src":"https://example.com/logo"},"owner_id":"proj_Test123"})
}

#[tokio::test]
async fn override_operations_encode_routes_and_updates() {
    let harness = Harness::script(
        static_config(),
        vec![
            Ok(response(200, json!({"data":[],"page_info":{"count":0}}))),
            Ok(response(201, override_value("ao_Test123"))),
            Ok(response(200, override_value("ao_Test123"))),
            Ok(response(200, override_value("ao_Other"))),
            Ok(response(204, json!(null))),
        ],
    );
    harness
        .client
        .app_overrides()
        .list(ListAppOverridesRequest::default())
        .await
        .unwrap();
    harness
        .client
        .app_overrides()
        .create(CreateAppOverrideRequest {
            app: "github".into(),
            name: "Override".into(),
            oauth_app_id: None,
            cfmap: None,
        })
        .await
        .unwrap();
    let id = AppOverrideId::new("ao_Test123").unwrap();
    harness
        .client
        .app_overrides()
        .get(id.clone())
        .await
        .unwrap();
    harness
        .client
        .app_overrides()
        .update(
            id.clone(),
            UpdateAppOverrideRequest {
                name: Some("Updated".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    harness.client.app_overrides().delete(id).await.unwrap();
    assert_wire(
        &harness.request(0),
        http::Method::GET,
        "https://api.pipedream.com/v1/connect/proj_Test123/app_overrides?limit=100",
        &[
            ("authorization", "Bearer static-secret"),
            ("x-pd-environment", "development"),
            ("accept", "application/json"),
        ],
        None,
    );
    harness.assert_requests(5);
}

#[tokio::test]
async fn override_get_checks_identity() {
    let harness = Harness::script(
        static_config(),
        vec![Ok(response(200, override_value("ao_Other")))],
    );
    let result = harness
        .client
        .app_overrides()
        .get(AppOverrideId::new("ao_Test123").unwrap())
        .await;
    assert!(matches!(result, Err(Error::Internal(_))));
    harness.assert_requests(1);
}
