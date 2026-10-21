//! Registry trigger queries and checked versioned retrieval.

use pipedream_connect_types::{
    AppSlug, ComponentKey, Cursor, GetComponentRequest, ListRegistryRequest, PageRequest, PropKind,
    RegistryKind,
};
use serde_json::json;

use crate::error::Error;
use crate::resources::test_support::{assert_get, component, page};
use crate::tests::support::{Harness, config, response};

#[tokio::test]
async fn list_uses_registry_trigger_filters_without_a_component_type_query() {
    let harness = Harness::new(vec![
        response(200, page(component())),
        response(200, page(component())),
    ]);
    let result = harness
        .client
        .triggers()
        .list(ListRegistryRequest {
            page: PageRequest {
                limit: 17,
                after: Some(Cursor::new("cursor+1").unwrap()),
                before: None,
            },
            query: Some("new & changed".into()),
            app: Some(AppSlug::new("google_calendar").unwrap()),
            registry: Some(RegistryKind::Private),
        })
        .await
        .unwrap();
    assert_get(
        harness.request(1),
        "https://api.pipedream.com/v1/connect/proj_Test123/triggers?after=cursor%2B1&app=google_calendar&limit=17&q=new+%26+changed&registry=private",
    );
    assert!(matches!(
        result.data[0].configurable_props[0].kind,
        PropKind::Other { .. }
    ));
    assert_eq!(result.extra["future_page"], json!({"keep":true}));
    harness
        .client
        .triggers()
        .list(ListRegistryRequest::default())
        .await
        .unwrap();
    assert_get(
        harness.request(2),
        "https://api.pipedream.com/v1/connect/proj_Test123/triggers?limit=100",
    );
    harness.assert_requests(3);
}

#[tokio::test]
async fn get_checks_keys_and_keeps_version_queries_and_unknown_envelope_fields() {
    let harness = Harness::new(vec![
        response(200, json!({"data":component(),"future_envelope":true})),
        response(200, json!({"data":component()})),
    ]);
    let result = harness
        .client
        .triggers()
        .get(
            ComponentKey::new("google_calendar-list-events").unwrap(),
            GetComponentRequest {
                version: Some("2.0.0+trigger".into()),
            },
        )
        .await
        .unwrap();
    assert_get(
        harness.request(1),
        "https://api.pipedream.com/v1/connect/proj_Test123/triggers/google_calendar-list-events?version=2.0.0%2Btrigger",
    );
    assert_eq!(result.extra["future_envelope"], true);
    assert!(matches!(
        harness
            .client
            .triggers()
            .get(
                ComponentKey::new("other").unwrap(),
                GetComponentRequest::default()
            )
            .await,
        Err(Error::Internal(_))
    ));
    harness.assert_requests(3);
}

#[tokio::test]
async fn invalid_pages_send_nothing_and_provider_failures_remain_typed() {
    let harness = Harness::script(config(), vec![]);
    assert!(matches!(
        harness
            .client
            .triggers()
            .list(ListRegistryRequest {
                page: PageRequest {
                    limit: 0,
                    ..PageRequest::default()
                },
                ..ListRegistryRequest::default()
            })
            .await,
        Err(Error::InvalidRequest(_))
    ));
    harness.assert_requests(0);
    let harness = Harness::new(vec![response(404, json!({"error":"missing"}))]);
    assert!(matches!(
        harness
            .client
            .triggers()
            .get(
                ComponentKey::new("google_calendar-list-events").unwrap(),
                GetComponentRequest::default()
            )
            .await,
        Err(Error::NotFound { .. })
    ));
    harness.assert_requests(2);
}
