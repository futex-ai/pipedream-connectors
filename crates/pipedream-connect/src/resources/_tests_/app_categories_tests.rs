//! Bare category responses, exact routes, identity checks, and shared errors.

use std::error::Error as StdError;

use pipedream_connect_types::AppCategoryId;
use serde_json::json;

use crate::error::Error;
use crate::resources::test_support::assert_get;
use crate::tests::support::{Harness, response};

#[tokio::test]
async fn list_decodes_a_bare_array_without_query_parameters() {
    let harness = Harness::new(vec![response(
        200,
        json!([{"id":"appcat_Calendar","name":"Calendar"}]),
    )]);
    let categories = harness.client.app_categories().list().await.unwrap();
    assert_eq!(categories.len(), 1);
    assert_eq!(
        categories[0].id.as_ref().unwrap().as_str(),
        "appcat_Calendar"
    );
    assert_get(
        harness.request(1),
        "https://api.pipedream.com/v1/connect/app_categories",
    );
    harness.assert_requests(2);
}

#[tokio::test]
async fn get_decodes_the_bare_category_and_preserves_unknown_fields() {
    let harness = Harness::new(vec![response(
        200,
        json!({"id":"appcat_Calendar","name":"Calendar","description":null,"future":{"keep":true}}),
    )]);
    let result = harness
        .client
        .app_categories()
        .get(AppCategoryId::new("appcat_Calendar").unwrap())
        .await
        .unwrap();
    assert_eq!(result.id.unwrap().as_str(), "appcat_Calendar");
    assert_eq!(result.extra["future"], json!({"keep":true}));
    assert_get(
        harness.request(1),
        "https://api.pipedream.com/v1/connect/app_categories/appcat_Calendar",
    );
    harness.assert_requests(2);
}

#[tokio::test]
async fn mismatched_or_absent_identities_are_internal_errors() {
    for reply in [json!({"id":"appcat_Other"}), json!({"name":"No identity"})] {
        let harness = Harness::new(vec![response(200, reply)]);
        let error = harness
            .client
            .app_categories()
            .get(AppCategoryId::new("appcat_Calendar").unwrap())
            .await
            .unwrap_err();
        let Error::Internal(source) = error else {
            panic!("expected identity failure")
        };
        assert!(source.source().is_some());
        harness.assert_requests(2);
    }
}

#[tokio::test]
async fn get_uses_shared_status_mapping() {
    let harness = Harness::new(vec![response(404, json!({"error":"not found"}))]);
    assert!(matches!(
        harness
            .client
            .app_categories()
            .get(AppCategoryId::new("appcat_Calendar").unwrap())
            .await,
        Err(Error::NotFound { .. })
    ));
    harness.assert_requests(2);
}
