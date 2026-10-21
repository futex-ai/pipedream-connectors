//! Downstream callers can use every resource mock through an object-safe trait.

#![cfg(feature = "unimock")]

use std::sync::Arc;

use pipedream_connect::error::Error;
use pipedream_connect::resources::actions::{ActionsApi, ActionsApiMock};
use pipedream_connect::resources::app_categories::{AppCategoriesApi, AppCategoriesApiMock};
use pipedream_connect::resources::apps::{AppsApi, AppsApiMock};
use pipedream_connect::resources::components::{ComponentsApi, ComponentsApiMock};
use pipedream_connect::resources::triggers::{TriggersApi, TriggersApiMock};
use pipedream_connect_types::{AppCategoryId, AppSlug, ComponentKey, GetComponentRequest};
use unimock::{MockFn, Unimock, matching};

#[tokio::test]
async fn all_resource_groups_export_usable_trait_object_mocks() {
    let apps: Arc<dyn AppsApi + Send + Sync> = Arc::new(Unimock::new(
        AppsApiMock::get
            .next_call(matching!(_))
            .returns(Err(Error::NotFound { message: None })),
    ));
    assert!(matches!(
        apps.get(AppSlug::new("google_calendar").unwrap()).await,
        Err(Error::NotFound { .. })
    ));
    let categories: Arc<dyn AppCategoriesApi + Send + Sync> = Arc::new(Unimock::new(
        AppCategoriesApiMock::get
            .next_call(matching!(_))
            .returns(Err(Error::NotFound { message: None })),
    ));
    assert!(matches!(
        categories
            .get(AppCategoryId::new("appcat_Calendar").unwrap())
            .await,
        Err(Error::NotFound { .. })
    ));
    let components: Arc<dyn ComponentsApi + Send + Sync> = Arc::new(Unimock::new(
        ComponentsApiMock::get
            .next_call(matching!(_, _))
            .returns(Err(Error::NotFound { message: None })),
    ));
    let key = ComponentKey::new("google_calendar-list-events").unwrap();
    assert!(matches!(
        components
            .get(key.clone(), GetComponentRequest::default())
            .await,
        Err(Error::NotFound { .. })
    ));
    let actions: Arc<dyn ActionsApi + Send + Sync> = Arc::new(Unimock::new(
        ActionsApiMock::get
            .next_call(matching!(_, _))
            .returns(Err(Error::NotFound { message: None })),
    ));
    assert!(matches!(
        actions
            .get(key.clone(), GetComponentRequest::default())
            .await,
        Err(Error::NotFound { .. })
    ));
    let triggers: Arc<dyn TriggersApi + Send + Sync> = Arc::new(Unimock::new(
        TriggersApiMock::get
            .next_call(matching!(_, _))
            .returns(Err(Error::NotFound { message: None })),
    ));
    assert!(matches!(
        triggers.get(key, GetComponentRequest::default()).await,
        Err(Error::NotFound { .. })
    ));
}
