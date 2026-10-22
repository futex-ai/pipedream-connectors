//! Action query encoding, identity checks, and early local validation.

use std::error::Error as StdError;

use pipedream_connect_types::{
    AppSlug, ComponentKey, Cursor, GetComponentRequest, ListRegistryRequest, PageRequest,
    RegistryKind,
};
use serde_json::json;

use crate::error::Error;
use crate::resources::test_support::{assert_get, component, page};
use crate::tests::support::{Harness, config, response};

#[tokio::test]
async fn list_encodes_all_action_filters_without_a_component_type_query() {
    let harness = Harness::new(vec![
        response(200, page(component())),
        response(200, page(component())),
    ]);
    let result = harness
        .client
        .actions()
        .list(ListRegistryRequest {
            page: PageRequest {
                limit: 11,
                after: None,
                before: Some(Cursor::new("cursor+1").unwrap()),
            },
            query: Some("list & read".into()),
            app: Some(AppSlug::new("google_calendar").unwrap()),
            registry: Some(RegistryKind::Public),
        })
        .await
        .unwrap();
    assert_get(
        harness.request(1),
        "https://api.pipedream.com/v1/connect/proj_Test123/actions?app=google_calendar&before=cursor%2B1&limit=11&q=list+%26+read&registry=public",
    );
    assert_eq!(result.data.len(), 1);
    assert_eq!(
        serde_json::to_value(&result.data[0].configurable_props[0]).unwrap(),
        component()["configurable_props"][0]
    );
    harness
        .client
        .actions()
        .list(ListRegistryRequest::default())
        .await
        .unwrap();
    assert_get(
        harness.request(2),
        "https://api.pipedream.com/v1/connect/proj_Test123/actions?limit=100",
    );
    harness.assert_requests(3);
}

#[tokio::test]
async fn get_checks_action_keys_and_preserves_the_version_and_response_envelope() {
    let harness = Harness::new(vec![
        response(200, json!({"data":component(),"future_envelope":true})),
        response(200, json!({"data":component()})),
    ]);
    let result = harness
        .client
        .actions()
        .get(
            ComponentKey::new("google_calendar-list-events").unwrap(),
            GetComponentRequest {
                version: Some("1.2.3+action".into()),
            },
        )
        .await
        .unwrap();
    assert_get(
        harness.request(1),
        "https://api.pipedream.com/v1/connect/proj_Test123/actions/google_calendar-list-events?version=1.2.3%2Baction",
    );
    assert_eq!(result.extra["future_envelope"], true);
    let error = harness
        .client
        .actions()
        .get(
            ComponentKey::new("other").unwrap(),
            GetComponentRequest::default(),
        )
        .await
        .unwrap_err();
    let Error::Internal(source) = error else {
        panic!("expected identity failure")
    };
    assert!(source.source().is_some());
    harness.assert_requests(3);
}

#[tokio::test]
async fn conflicting_cursors_send_nothing_and_provider_failures_use_shared_mapping() {
    let harness = Harness::script(config(), vec![]);
    assert!(matches!(
        harness
            .client
            .actions()
            .list(ListRegistryRequest {
                page: PageRequest {
                    after: Some(Cursor::new("after").unwrap()),
                    before: Some(Cursor::new("before").unwrap()),
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
            .actions()
            .get(
                ComponentKey::new("google_calendar-list-events").unwrap(),
                GetComponentRequest::default()
            )
            .await,
        Err(Error::NotFound { .. })
    ));
    harness.assert_requests(2);
}
