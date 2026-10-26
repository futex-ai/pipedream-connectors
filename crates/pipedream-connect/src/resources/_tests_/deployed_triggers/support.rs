//! Shared deployed-trigger request and response fixtures.

use pipedream_connect_types::{DeployedTriggerId, DeployedTriggerQuery, ExternalUserId, WebhookId};
use serde_json::{Value, json};

pub(super) fn trigger_id() -> DeployedTriggerId {
    DeployedTriggerId::new("dc_Trigger123").expect("valid test trigger ID")
}

pub(super) fn webhook_id() -> WebhookId {
    WebhookId::new("wh_Webhook123").expect("valid test webhook ID")
}

pub(super) fn query() -> DeployedTriggerQuery {
    DeployedTriggerQuery {
        external_user_id: ExternalUserId::new("customer_1").expect("valid test user ID"),
    }
}

pub(super) fn deployed_trigger(id: &str) -> Value {
    json!({
        "id": id,
        "owner_id": "owner_1",
        "component_id": "component_1",
        "component_key": "google_calendar-new-event",
        "configurable_props": [],
        "configured_props": {},
        "active": true,
        "created_at": 1_000_000,
        "updated_at": 1_001_000,
        "name": "Calendar trigger",
        "name_slug": "calendar-trigger",
        "callback_observations": null,
        "emit_on_deploy": false,
        "type": "DeployedComponent",
        "webhook_signing_key": "deployment-secret",
        "webhook_urls": ["http://127.0.0.1:50151/events"],
        "webhooks": []
    })
}

pub(super) fn page(data: Vec<Value>) -> Value {
    json!({"data": data, "page_info": {"count": 4, "end_cursor": "next_cursor"}})
}
