//! App query filters, identity checks, and lossless response mapping.

use http::HeaderValue;
use pipedream_connect_types::{
    AppAuthType, AppCategoryId, AppSlug, AppSortKey, Cursor, ListAppsRequest, PageRequest,
    SortDirection,
};
use serde_json::json;
use std::error::Error as StdError;
use std::num::NonZeroU32;
use std::time::Duration;

use crate::error::Error;
use crate::resources::test_support::{app, assert_get, page};
use crate::retry::RetryPolicy;
use crate::tests::support::{Harness, config, response};

#[tokio::test]
async fn list_encodes_every_filter_and_keeps_unknown_values_and_page_metadata() {
    let harness = Harness::new(vec![response(200, page(app()))]);
    let result = harness
        .client
        .apps()
        .list(ListAppsRequest {
            page: PageRequest {
                limit: 23,
                after: Some(Cursor::new("next+cursor").unwrap()),
                before: None,
            },
            query: Some("calendar & schedule".into()),
            sort_key: Some(AppSortKey::NameSlug),
            sort_direction: Some(SortDirection::Desc),
            category_ids: vec![
                AppCategoryId::new("appcat_One").unwrap(),
                AppCategoryId::new("appcat_Two").unwrap(),
            ],
            has_components: Some(true),
            has_actions: Some(false),
            has_triggers: Some(true),
        })
        .await
        .unwrap();
    assert_get(
        harness.request(1),
        "https://api.pipedream.com/v1/connect/apps?after=next%2Bcursor&category_ids=appcat_One&category_ids=appcat_Two&has_actions=false&has_components=true&has_triggers=true&limit=23&q=calendar+%26+schedule&sort_direction=desc&sort_key=name_slug",
    );
    assert_eq!(
        result.data[0].auth_type,
        Some(AppAuthType::Other("future_auth".into()))
    );
    let encoded = serde_json::to_value(result).unwrap();
    assert_eq!(encoded["data"][0]["future"], json!({"keep":true}));
    assert_eq!(encoded["page_info"]["future_count"], 42);
    assert_eq!(encoded["future_page"], json!({"keep":true}));
    harness.assert_requests(2);
}

#[tokio::test]
async fn default_and_future_sort_queries_do_not_add_local_defaults() {
    let harness = Harness::new(vec![response(200, page(app())), response(200, page(app()))]);
    harness
        .client
        .apps()
        .list(ListAppsRequest::default())
        .await
        .unwrap();
    assert_get(
        harness.request(1),
        "https://api.pipedream.com/v1/connect/apps?limit=100",
    );
    harness
        .client
        .apps()
        .list(ListAppsRequest {
            sort_key: Some(AppSortKey::Other("future+rank".into())),
            sort_direction: Some(SortDirection::Other("future".into())),
            ..ListAppsRequest::default()
        })
        .await
        .unwrap();
    assert_get(
        harness.request(2),
        "https://api.pipedream.com/v1/connect/apps?limit=100&sort_direction=future&sort_key=future%2Brank",
    );
    harness.assert_requests(3);
}

#[tokio::test]
async fn get_checks_the_slug_and_preserves_the_complete_response_envelope() {
    let harness = Harness::new(vec![response(
        200,
        json!({"data":app(),"future_envelope":[1,2]}),
    )]);
    let result = harness
        .client
        .apps()
        .get(AppSlug::new("google_calendar").unwrap())
        .await
        .unwrap();
    assert_get(
        harness.request(1),
        "https://api.pipedream.com/v1/connect/apps/google_calendar",
    );
    assert_eq!(result.data.name_slug.as_str(), "google_calendar");
    assert_eq!(result.extra["future_envelope"], json!([1, 2]));
    harness.assert_requests(2);
}

#[tokio::test]
async fn get_accepts_a_returned_app_id() {
    let harness = Harness::new(vec![response(
        200,
        json!({"data":{"id":"app_OkrhR1","name_slug":"google_calendar","name":"Google Calendar","img_src":"https://example.com/logo"}}),
    )]);
    let result = harness
        .client
        .apps()
        .get(AppSlug::new("app_OkrhR1").unwrap())
        .await
        .unwrap();
    assert_eq!(result.data.id.as_deref(), Some("app_OkrhR1"));
    harness.assert_requests(2);
}

#[tokio::test]
async fn get_uses_the_configured_get_retry_policy() {
    let harness = Harness::script(
        config().with_retry_policy(RetryPolicy::new(
            NonZeroU32::new(2).unwrap(),
            Duration::from_secs(10),
            Duration::ZERO,
        )),
        vec![
            Ok(crate::tests::support::token("access-secret", 3600)),
            Ok(response(503, json!({"error":"temporary"}))),
            Ok(response(200, json!({"data":app()}))),
        ],
    )
    .record_sleeps();
    harness
        .client
        .apps()
        .get(AppSlug::new("google_calendar").unwrap())
        .await
        .unwrap();
    assert_eq!(harness.requests.lock().unwrap().len(), 3);
    assert_eq!(harness.sleeps.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn wrong_identities_and_http_failures_return_the_shared_error_contract() {
    let mut wrong = app();
    wrong["name_slug"] = json!("another_app");
    let mut limited = response(429, json!({"error":"throttled"}));
    limited
        .headers
        .insert("retry-after", HeaderValue::from_static("5"));
    let harness = Harness::new(vec![
        response(200, json!({"data":wrong})),
        response(404, json!({"error":"missing"})),
        limited,
    ]);
    let error = harness
        .client
        .apps()
        .get(AppSlug::new("google_calendar").unwrap())
        .await
        .unwrap_err();
    let Error::Internal(source) = error else {
        panic!("expected identity failure")
    };
    assert!(source.source().is_some());
    assert!(matches!(
        harness
            .client
            .apps()
            .get(AppSlug::new("missing").unwrap())
            .await,
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(
        harness.client.apps().list(ListAppsRequest::default()).await,
        Err(Error::RateLimited { .. })
    ));
    harness.assert_requests(4);
}

#[tokio::test]
async fn invalid_local_pages_send_no_token_or_connect_request() {
    let harness = Harness::script(config(), vec![]);
    let result = harness
        .client
        .apps()
        .list(ListAppsRequest {
            page: PageRequest {
                limit: 0,
                ..PageRequest::default()
            },
            ..ListAppsRequest::default()
        })
        .await;
    assert!(matches!(result, Err(Error::InvalidRequest(_))));
    harness.assert_requests(0);
}
