//! Event, workflow, and webhook destination tests.

use pipedream_connect_types::{
    ListTriggerEventsRequest, UpdateTriggerWebhooksRequest, UpdateTriggerWorkflowsRequest,
    WorkflowEndpoint,
};
use serde_json::json;

use crate::resources::test_support::assert_wire;
use crate::tests::support::{Harness, response};

use super::support::{query, trigger_id, webhook_id};

#[tokio::test]
async fn events_workflows_and_webhooks_preserve_timestamps_and_signing_keys() {
    let id = trigger_id();
    let webhook = webhook_id();
    let harness = Harness::new(vec![
        response(
            200,
            json!({"data": [{"e":{"title":"created"},"k":"emit","ts":1_000_000,"id":"event_1","sum":"Created"}]}),
        ),
        response(200, json!({"workflow_ids":["p_workflow1"]})),
        response(
            200,
            json!({"webhook_urls":["https://example.com/events"],"webhooks":[{"id":"wh_Webhook123","url":"https://example.com/events","signing_key":"webhook-secret","signing_key_set":true}]}),
        ),
        response(
            200,
            json!({"data":{"id":"wh_Webhook123","url":"https://example.com/events","signing_key":"webhook-secret","signing_key_set":true,"created_at":1000,"updated_at":1001}}),
        ),
        response(
            200,
            json!({"data":{"id":"wh_Webhook123","url":"https://example.com/events","signing_key":"new-secret","signing_key_set":true,"created_at":1000,"updated_at":1001}}),
        ),
    ]);
    let events = harness
        .client
        .deployed_triggers()
        .list_events(
            id.clone(),
            ListTriggerEventsRequest::new(query().external_user_id.clone(), 10).unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        events.data[0].timestamp().unwrap().timestamp_millis(),
        1_000_000
    );
    let workflows = harness
        .client
        .deployed_triggers()
        .get_workflows(id.clone(), query())
        .await
        .unwrap();
    assert_eq!(workflows.workflow_ids[0].as_str(), "p_workflow1");
    let webhooks = harness
        .client
        .deployed_triggers()
        .list_webhooks(id.clone(), query())
        .await
        .unwrap();
    assert_eq!(
        webhooks.webhooks.as_ref().unwrap()[0]
            .signing_key
            .as_ref()
            .unwrap()
            .expose_secret(),
        "webhook-secret"
    );
    assert!(!format!("{webhooks:?}").contains("webhook-secret"));
    let retrieved = harness
        .client
        .deployed_triggers()
        .get_webhook(id.clone(), webhook.clone(), query())
        .await
        .unwrap();
    assert_eq!(retrieved.data.signing_key.expose_secret(), "webhook-secret");
    assert!(!format!("{retrieved:?}").contains("webhook-secret"));
    let regenerated = harness
        .client
        .deployed_triggers()
        .regenerate_webhook_signing_key(id, webhook, query())
        .await
        .unwrap();
    assert_eq!(regenerated.data.signing_key.expose_secret(), "new-secret");
    assert!(!format!("{regenerated:?}").contains("new-secret"));
    assert!(
        harness
            .request(1)
            .url
            .expose_secret()
            .contains("/events?external_user_id=customer_1&n=10")
    );
    for index in 1..=5 {
        assert!(
            harness
                .request(index)
                .url
                .expose_secret()
                .contains("external_user_id=customer_1")
        );
    }
    harness.assert_requests(6);
}

#[tokio::test]
async fn workflow_and_webhook_updates_use_tenant_queries_and_json_bodies() {
    let id = trigger_id();
    let harness = Harness::new(vec![
        response(200, json!({"workflow_ids":["p_workflow1"]})),
        response(
            200,
            json!({"webhook_urls":["http://127.0.0.1:50151/events"]}),
        ),
    ]);
    harness
        .client
        .deployed_triggers()
        .set_workflows(
            id.clone(),
            query(),
            UpdateTriggerWorkflowsRequest {
                workflow_ids: vec![
                    WorkflowEndpoint::new("p_workflow1").expect("valid workflow ID"),
                ],
            },
        )
        .await
        .unwrap();
    harness
        .client
        .deployed_triggers()
        .set_webhooks(
            id,
            query(),
            UpdateTriggerWebhooksRequest {
                webhook_urls: vec!["http://127.0.0.1:50151/events".into()],
            },
        )
        .await
        .unwrap();
    assert_wire(
        &harness.request(1),
        http::Method::PUT,
        "https://api.pipedream.com/v1/connect/proj_Test123/deployed-triggers/dc_Trigger123/pipelines?external_user_id=customer_1",
        &[
            ("authorization", "Bearer access-secret"),
            ("x-pd-environment", "development"),
            ("accept", "application/json"),
            ("content-type", "application/json"),
        ],
        Some(&json!({"workflow_ids":["p_workflow1"]})),
    );
    assert_wire(
        &harness.request(2),
        http::Method::PUT,
        "https://api.pipedream.com/v1/connect/proj_Test123/deployed-triggers/dc_Trigger123/webhooks?external_user_id=customer_1",
        &[
            ("authorization", "Bearer access-secret"),
            ("x-pd-environment", "development"),
            ("accept", "application/json"),
            ("content-type", "application/json"),
        ],
        Some(&json!({"webhook_urls":["http://127.0.0.1:50151/events"]})),
    );
    for index in 1..=2 {
        assert!(
            harness
                .request(index)
                .url
                .expose_secret()
                .contains("external_user_id=customer_1")
        );
    }
    harness.assert_requests(3);
}
