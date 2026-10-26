//! Project webhook request and signing-key redaction tests.

use pipedream_connect_types::SetWebhookRequest;
use serde_json::json;

use crate::resources::test_support::{ExpectedBody, assert_wire};
use crate::tests::support::{Harness, response, static_config};

#[tokio::test]
async fn project_webhook_operations_keep_wire_envelopes_and_redact_keys() {
    let harness = Harness::script(
        static_config(),
        vec![
            Ok(response(200, json!({"data":null}))),
            Ok(response(
                200,
                json!({"data":{"id":"wh_Test","url":"https://example.com","signing_key_set":true,"created_at":1,"updated_at":2,"signing_key":"set-secret"}}),
            )),
            Ok(response(204, json!(null))),
            Ok(response(
                200,
                json!({"data":{"id":"wh_Test","url":"https://example.com","signing_key_set":true,"created_at":1,"updated_at":2,"signing_key":"regen-secret"}}),
            )),
        ],
    );
    harness.client.project_webhooks().get().await.unwrap();
    let set = harness
        .client
        .project_webhooks()
        .set(SetWebhookRequest {
            url: "https://example.com".into(),
        })
        .await
        .unwrap();
    assert!(!format!("{:?}", set).contains("set-secret"));
    harness.client.project_webhooks().delete().await.unwrap();
    let regenerated = harness
        .client
        .project_webhooks()
        .regenerate_signing_key()
        .await
        .unwrap();
    assert!(!format!("{:?}", regenerated).contains("regen-secret"));
    assert_wire(
        &harness.request(1),
        http::Method::PUT,
        "https://api.pipedream.com/v1/connect/proj_Test123/webhook",
        &[
            ("authorization", "Bearer static-secret"),
            ("x-pd-environment", "development"),
            ("accept", "application/json"),
            ("content-type", "application/json"),
        ],
        Some(ExpectedBody::Json(&json!({"url":"https://example.com"}))),
    );
    harness.assert_requests(4);
}
