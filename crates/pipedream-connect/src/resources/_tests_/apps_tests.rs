//! App query filters, identity checks, and lossless response mapping.

use http::HeaderValue;
use pipedream_connect_types::{
    AppAuthType, AppCategoryId, AppSlug, AppSortKey, Cursor, ListAppsRequest, PageRequest,
    SortDirection,
};
use serde_json::json;

use crate::error::Error;
use crate::resources::test_support::{app, assert_get, page};
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
        "https://api.pipedream.com/v1/connect/apps?limit=23&after=next%2Bcursor&q=calendar+%26+schedule&sort_key=name_slug&sort_direction=desc&category_ids=appcat_One&category_ids=appcat_Two&has_components=true&has_actions=false&has_triggers=true",
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
        "https://api.pipedream.com/v1/connect/apps?limit=100&sort_key=future%2Brank&sort_direction=future",
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
    assert!(matches!(
        harness
            .client
            .apps()
            .get(AppSlug::new("google_calendar").unwrap())
            .await,
        Err(Error::Internal(_))
    ));
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
