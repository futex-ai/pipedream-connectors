//! Deployment, listing, and deletion lifecycle tests.

use pipedream_connect_types::{
    Cursor, DeleteTriggerRequest, Emitter, EmitterType, ExternalUserId,
    ListDeployedTriggersRequest, PageRequest, UpdateTriggerRequest,
};
use serde_json::json;

use crate::error::Error;
use crate::resources::test_support::assert_get;
use crate::tests::support::{Harness, response};

use super::support::{deployed_trigger, page, query, request_body, trigger_id};

#[tokio::test]
async fn deploy_sends_the_external_user_and_callback_options() {
    let harness = Harness::new(vec![response(
        200,
        json!({"data": deployed_trigger("dc_Trigger123"), "future": true}),
    )]);
    let request = serde_json::from_value(json!({
        "id": "google_calendar-new-event",
        "version": "1.0.0",
        "external_user_id": "customer_1",
        "configured_props": {},
        "dynamic_props_id": "dynamic_1",
        "workflow_id": "p_workflow1",
        "webhook_url": "http://127.0.0.1:50151/events",
        "emit_on_deploy": true
    }))
    .expect("valid deployment request");
    let result = harness.client.triggers().deploy(request).await.unwrap();
    assert!(matches!(result.data, Emitter::DeployedComponent(_)));
    assert!(!format!("{result:?}").contains("deployment-secret"));
    assert_eq!(result.extra["future"], true);
    let sent = harness.request(1);
    assert_eq!(
        sent.url.expose_secret(),
        "https://api.pipedream.com/v1/connect/proj_Test123/triggers/deploy"
    );
    assert_eq!(request_body(sent)["external_user_id"], "customer_1");
    assert_eq!(request_body(harness.request(1))["emit_on_deploy"], true);
    harness.assert_requests(2);
}

#[tokio::test]
async fn list_keeps_every_emitter_variant_and_scopes_the_tenant() {
    let http = json!({
        "id": "hi_Interface123", "key": "http", "endpoint_url": "https://example.com",
        "custom_response": false, "created_at": 1_000_000, "updated_at": 1_001_000,
        "type": "HttpInterface"
    });
    let timer = json!({
        "id": "ti_Timer123", "interval_seconds": 60, "cron": null, "timezone": "UTC",
        "schedule_changed_at": 1_000_000, "created_at": 1_000_000,
        "updated_at": 1_001_000, "type": "TimerInterface"
    });
    let other = json!({"id": "future_1", "type": "FutureInterface", "future": true});
    let harness = Harness::new(vec![response(
        200,
        page(vec![deployed_trigger("dc_Trigger123"), http, timer, other]),
    )]);
    let result = harness
        .client
        .deployed_triggers()
        .list(ListDeployedTriggersRequest {
            external_user_id: ExternalUserId::new("customer_1").expect("valid test user ID"),
            page: PageRequest {
                limit: 4,
                after: Some(Cursor::new("after_cursor").expect("valid test cursor")),
                before: None,
            },
            emitter_type: Some(EmitterType::Source),
        })
        .await
        .unwrap();
    assert!(matches!(result.data[0], Emitter::DeployedComponent(_)));
    assert!(matches!(result.data[1], Emitter::HttpInterface(_)));
    assert!(matches!(result.data[2], Emitter::TimerInterface(_)));
    assert!(matches!(result.data[3], Emitter::Other(_)));
    let Emitter::DeployedComponent(component) = &result.data[0] else {
        unreachable!("checked deployed component emitter")
    };
    assert_eq!(component.created_at.timestamp_millis(), 1_000_000);
    let Emitter::HttpInterface(interface) = &result.data[1] else {
        unreachable!("checked HTTP emitter")
    };
    assert_eq!(interface.created_at.timestamp_millis(), 1_000_000);
    let Emitter::TimerInterface(timer) = &result.data[2] else {
        unreachable!("checked timer emitter")
    };
    assert_eq!(timer.schedule_changed_at.timestamp_millis(), 1_000_000);
    assert_get(
        harness.request(1),
        "https://api.pipedream.com/v1/connect/proj_Test123/deployed-triggers?after=after_cursor&emitter_type=source&external_user_id=customer_1&limit=4",
    );
    harness.assert_requests(2);
}

#[tokio::test]
async fn crud_calls_keep_tenant_scope_and_delete_can_ignore_missing_deployments() {
    let envelope = json!({"data": deployed_trigger("dc_Trigger123")});
    let harness = Harness::new(vec![
        response(200, envelope.clone()),
        response(200, json!({"data": deployed_trigger("dc_Other123")})),
        response(404, json!({"error": "missing"})),
        response(404, json!({"error": "missing"})),
    ]);
    let id = trigger_id();
    harness
        .client
        .deployed_triggers()
        .get(id.clone(), query())
        .await
        .unwrap();
    harness
        .client
        .deployed_triggers()
        .update(
            id.clone(),
            query(),
            UpdateTriggerRequest {
                active: Some(false),
                name: Some("Updated".into()),
                ..UpdateTriggerRequest::default()
            },
        )
        .await
        .unwrap();
    harness
        .client
        .deployed_triggers()
        .delete(
            id.clone(),
            DeleteTriggerRequest {
                external_user_id: query().external_user_id.clone(),
                ignore_hook_errors: Some(true),
                missing_ok: true,
            },
        )
        .await
        .unwrap();
    let result = harness
        .client
        .deployed_triggers()
        .delete(
            id,
            DeleteTriggerRequest {
                external_user_id: query().external_user_id,
                ignore_hook_errors: Some(false),
                missing_ok: false,
            },
        )
        .await;
    assert!(matches!(result, Err(Error::NotFound { .. })));
    assert_eq!(
        request_body(harness.request(2)),
        json!({"active":false,"name":"Updated"})
    );
    for index in 1..=4 {
        assert!(
            harness
                .request(index)
                .url
                .expose_secret()
                .contains("external_user_id=customer_1")
        );
    }
    harness.assert_requests(5);
}
