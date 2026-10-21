//! Component filters, version queries, identity checks, and lossless kinds.

use pipedream_connect_types::{
    AppSlug, ComponentKey, ComponentStash, ComponentType, Cursor, GetComponentRequest,
    ListComponentsRequest, PageRequest, PropKind, RegistryKind,
};
use serde_json::json;
use std::error::Error as StdError;

use crate::error::Error;
use crate::resources::test_support::{assert_get, component, page};
use crate::tests::support::{Harness, config, response};

#[tokio::test]
async fn list_encodes_every_filter_and_keeps_unknown_component_and_prop_kinds() {
    let harness = Harness::new(vec![response(200, page(component()))]);
    let result = harness
        .client
        .components()
        .list(ListComponentsRequest {
            page: PageRequest {
                limit: 12,
                after: None,
                before: Some(Cursor::new("prior+cursor").unwrap()),
            },
            query: Some("list & read".into()),
            app: Some(AppSlug::new("google_calendar").unwrap()),
            registry: Some(RegistryKind::All),
            component_type: Some(ComponentType::Action),
        })
        .await
        .unwrap();
    assert_get(
        harness.request(1),
        "https://api.pipedream.com/v1/connect/proj_Test123/components?app=google_calendar&before=prior%2Bcursor&component_type=action&limit=12&q=list+%26+read&registry=all",
    );
    assert_eq!(result.data.len(), 1);
    assert_eq!(result.page_info.count, Some(1));
    assert_eq!(
        result.data[0].component_type,
        Some(ComponentType::Other("future_component".into()))
    );
    assert_eq!(
        result.data[0].stash,
        Some(ComponentStash::Other("future_stash".into()))
    );
    let prop = &result.data[0].configurable_props[0];
    assert!(matches!(&prop.kind,PropKind::Other { type_name, .. } if type_name == "future_kind"));
    assert_eq!(
        serde_json::to_value(prop).unwrap(),
        component()["configurable_props"][0]
    );
    harness.assert_requests(2);
}

#[tokio::test]
async fn get_encodes_versions_checks_keys_and_keeps_the_data_envelope() {
    let harness = Harness::new(vec![
        response(200, json!({"data":component(),"future_envelope":true})),
        response(200, json!({"data":component()})),
    ]);
    let result = harness
        .client
        .components()
        .get(
            ComponentKey::new("google_calendar-list-events").unwrap(),
            GetComponentRequest {
                version: Some("1.2.3+build & next".into()),
            },
        )
        .await
        .unwrap();
    assert_eq!(result.extra["future_envelope"], true);
    assert_get(
        harness.request(1),
        "https://api.pipedream.com/v1/connect/proj_Test123/components/google_calendar-list-events?version=1.2.3%2Bbuild+%26+next",
    );
    let error = harness
        .client
        .components()
        .get(
            ComponentKey::new("different").unwrap(),
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
async fn default_and_future_enum_filters_use_the_model_wire_values() {
    let harness = Harness::new(vec![
        response(200, page(component())),
        response(200, page(component())),
    ]);
    harness
        .client
        .components()
        .list(ListComponentsRequest::default())
        .await
        .unwrap();
    assert_get(
        harness.request(1),
        "https://api.pipedream.com/v1/connect/proj_Test123/components?limit=100",
    );
    harness
        .client
        .components()
        .list(ListComponentsRequest {
            registry: Some(RegistryKind::Other("future+registry".into())),
            component_type: Some(ComponentType::Other("future_kind".into())),
            ..ListComponentsRequest::default()
        })
        .await
        .unwrap();
    assert_get(
        harness.request(2),
        "https://api.pipedream.com/v1/connect/proj_Test123/components?component_type=future_kind&limit=100&registry=future%2Bregistry",
    );
    harness.assert_requests(3);
}

#[tokio::test]
async fn invalid_pages_send_nothing_and_http_failures_keep_the_shared_category() {
    let harness = Harness::script(config(), vec![]);
    let result = harness
        .client
        .components()
        .list(ListComponentsRequest {
            page: PageRequest {
                limit: 201,
                ..PageRequest::default()
            },
            ..ListComponentsRequest::default()
        })
        .await;
    assert!(matches!(result, Err(Error::InvalidRequest(_))));
    harness.assert_requests(0);
    let harness = Harness::new(vec![response(403, json!({"error":"denied"}))]);
    assert!(matches!(
        harness
            .client
            .components()
            .get(
                ComponentKey::new("google_calendar-list-events").unwrap(),
                GetComponentRequest::default()
            )
            .await,
        Err(Error::Forbidden { .. })
    ));
    harness.assert_requests(2);
}
