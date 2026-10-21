//! Trigger callback and response identity validation tests.

use pipedream_connect_types::{ListTriggerEventsRequest, UpdateTriggerWebhooksRequest};
use serde_json::json;

use crate::error::Error;
use crate::tests::support::{Harness, config, response};

use super::support::{deployed_trigger, query, trigger_id, webhook_id};

#[tokio::test]
async fn event_history_rejects_an_out_of_range_limit_before_authentication() {
    let harness = Harness::script(config(), vec![]);
    let request = ListTriggerEventsRequest {
        external_user_id: query().external_user_id,
        limit: 101,
    };
    assert!(matches!(
        harness
            .client
            .deployed_triggers()
            .list_events(trigger_id(), request)
            .await,
        Err(Error::InvalidRequest(_))
    ));
    harness.assert_requests(0);
}

#[tokio::test]
async fn invalid_callback_urls_fail_before_authentication() {
    let harness = Harness::script(config(), vec![]);
    let request = serde_json::from_value(json!({
        "id": "google_calendar-new-event",
        "external_user_id": "customer_1",
        "webhook_url": "http://public.example/events"
    }))
    .expect("valid model shape");
    assert!(matches!(
        harness.client.triggers().deploy(request).await,
        Err(Error::InvalidRequest(_))
    ));
    assert!(matches!(
        harness
            .client
            .deployed_triggers()
            .set_webhooks(
                trigger_id(),
                query(),
                UpdateTriggerWebhooksRequest {
                    webhook_urls: vec!["https://user:secret@example.com".into()]
                },
            )
            .await,
        Err(Error::InvalidRequest(_))
    ));
    harness.assert_requests(0);
}

#[tokio::test]
async fn identity_checks_reject_wrong_trigger_and_webhook_responses() {
    let harness = Harness::new(vec![
        response(200, json!({"data": deployed_trigger("dc_Other123")})),
        response(
            200,
            json!({"data":{"id":"wh_Other123","url":"https://example.com","signing_key":"secret","signing_key_set":true,"created_at":1,"updated_at":2}}),
        ),
    ]);
    assert!(matches!(
        harness
            .client
            .deployed_triggers()
            .get(trigger_id(), query())
            .await,
        Err(Error::Internal(_))
    ));
    assert!(matches!(
        harness
            .client
            .deployed_triggers()
            .get_webhook(trigger_id(), webhook_id(), query())
            .await,
        Err(Error::Internal(_))
    ));
    harness.assert_requests(3);
}
