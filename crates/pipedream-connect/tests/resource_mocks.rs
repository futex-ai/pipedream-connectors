//! Downstream callers can use every resource mock through an object-safe trait.

#![cfg(feature = "unimock")]

use std::sync::Arc;

use pipedream_connect::error::Error;
use pipedream_connect::resources::actions::{ActionsApi, ActionsApiMock};
use pipedream_connect::resources::app_categories::{AppCategoriesApi, AppCategoriesApiMock};
use pipedream_connect::resources::apps::{AppsApi, AppsApiMock};
use pipedream_connect::resources::components::{ComponentsApi, ComponentsApiMock};
use pipedream_connect::resources::triggers::{TriggersApi, TriggersApiMock};
use pipedream_connect::resources::{ConnectResources, ConnectResourcesMock};
use pipedream_connect_types::{
    AppCategoryId, AppSlug, ComponentKey, ConfigurePropRequest, GetComponentRequest,
    ListAppsRequest, ListComponentsRequest, ListRegistryRequest, ReloadPropsRequest,
    RunActionRequest,
};
use unimock::{MockFn, Unimock, matching};

#[tokio::test]
async fn all_resource_groups_export_usable_trait_object_mocks() {
    let apps: Arc<dyn AppsApi + Send + Sync> = Arc::new(Unimock::new((
        AppsApiMock::list
            .next_call(matching!(_))
            .returns(Err(Error::NotFound { message: None })),
        AppsApiMock::get
            .next_call(matching!(_))
            .returns(Err(Error::NotFound { message: None })),
        AppsApiMock::get
            .next_call(matching!(_))
            .returns(Err(Error::NotFound { message: None })),
    )));
    assert!(matches!(
        apps.list(ListAppsRequest::default()).await,
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(
        apps.get(AppSlug::new("google_calendar").unwrap()).await,
        Err(Error::NotFound { .. })
    ));
    let categories: Arc<dyn AppCategoriesApi + Send + Sync> = Arc::new(Unimock::new((
        AppCategoriesApiMock::list
            .next_call(matching!())
            .returns(Err(Error::NotFound { message: None })),
        AppCategoriesApiMock::get
            .next_call(matching!(_))
            .returns(Err(Error::NotFound { message: None })),
    )));
    assert!(matches!(
        categories.list().await,
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(
        categories
            .get(AppCategoryId::new("appcat_Calendar").unwrap())
            .await,
        Err(Error::NotFound { .. })
    ));
    let components: Arc<dyn ComponentsApi + Send + Sync> = Arc::new(Unimock::new((
        ComponentsApiMock::list
            .next_call(matching!(_))
            .returns(Err(Error::NotFound { message: None })),
        ComponentsApiMock::get
            .next_call(matching!(_, _))
            .returns(Err(Error::NotFound { message: None })),
        ComponentsApiMock::configure
            .next_call(matching!(_))
            .returns(Err(Error::NotFound { message: None })),
        ComponentsApiMock::reload_props
            .next_call(matching!(_))
            .returns(Err(Error::NotFound { message: None })),
    )));
    assert!(matches!(
        components.list(ListComponentsRequest::default()).await,
        Err(Error::NotFound { .. })
    ));
    let key = ComponentKey::new("google_calendar-list-events").unwrap();
    assert!(matches!(
        components
            .get(key.clone(), GetComponentRequest::default())
            .await,
        Err(Error::NotFound { .. })
    ));
    let configure: ConfigurePropRequest = serde_json::from_value(serde_json::json!({
        "id":"google_calendar-list-events","external_user_id":"user","prop_name":"calendar"
    }))
    .unwrap();
    assert!(matches!(
        components.configure(configure.clone()).await,
        Err(Error::NotFound { .. })
    ));
    let reload: ReloadPropsRequest = serde_json::from_value(serde_json::json!({
        "id":"google_calendar-list-events","external_user_id":"user"
    }))
    .unwrap();
    assert!(matches!(
        components.reload_props(reload.clone()).await,
        Err(Error::NotFound { .. })
    ));
    let actions: Arc<dyn ActionsApi + Send + Sync> = Arc::new(Unimock::new((
        ActionsApiMock::list
            .next_call(matching!(_))
            .returns(Err(Error::NotFound { message: None })),
        ActionsApiMock::get
            .next_call(matching!(_, _))
            .returns(Err(Error::NotFound { message: None })),
        ActionsApiMock::configure
            .next_call(matching!(_))
            .returns(Err(Error::NotFound { message: None })),
        ActionsApiMock::reload_props
            .next_call(matching!(_))
            .returns(Err(Error::NotFound { message: None })),
        ActionsApiMock::run
            .next_call(matching!(_))
            .returns(Err(Error::NotFound { message: None })),
    )));
    assert!(matches!(
        actions.list(ListRegistryRequest::default()).await,
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(
        actions
            .get(key.clone(), GetComponentRequest::default())
            .await,
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(
        actions.configure(configure.clone()).await,
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(
        actions.reload_props(reload.clone()).await,
        Err(Error::NotFound { .. })
    ));
    let run: RunActionRequest = serde_json::from_value(serde_json::json!({
        "id":"google_calendar-list-events","external_user_id":"user"
    }))
    .unwrap();
    assert!(matches!(
        actions.run(run).await,
        Err(Error::NotFound { .. })
    ));
    let triggers: Arc<dyn TriggersApi + Send + Sync> = Arc::new(Unimock::new((
        TriggersApiMock::list
            .next_call(matching!(_))
            .returns(Err(Error::NotFound { message: None })),
        TriggersApiMock::get
            .next_call(matching!(_, _))
            .returns(Err(Error::NotFound { message: None })),
        TriggersApiMock::configure
            .next_call(matching!(_))
            .returns(Err(Error::NotFound { message: None })),
        TriggersApiMock::reload_props
            .next_call(matching!(_))
            .returns(Err(Error::NotFound { message: None })),
    )));
    assert!(matches!(
        triggers.list(ListRegistryRequest::default()).await,
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(
        triggers.get(key, GetComponentRequest::default()).await,
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(
        triggers.configure(configure).await,
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(
        triggers.reload_props(reload).await,
        Err(Error::NotFound { .. })
    ));

    let factory: Arc<dyn ConnectResources + Send + Sync> = Arc::new(Unimock::new((
        ConnectResourcesMock::apps
            .next_call(matching!())
            .returns(apps),
        ConnectResourcesMock::app_categories
            .next_call(matching!())
            .returns(categories),
        ConnectResourcesMock::components
            .next_call(matching!())
            .returns(components),
        ConnectResourcesMock::actions
            .next_call(matching!())
            .returns(actions),
        ConnectResourcesMock::triggers
            .next_call(matching!())
            .returns(triggers),
    )));
    assert!(matches!(
        factory
            .apps()
            .get(AppSlug::new("google_calendar").unwrap())
            .await,
        Err(Error::NotFound { .. })
    ));
    let _ = factory.app_categories();
    let _ = factory.components();
    let _ = factory.actions();
    let _ = factory.triggers();
}
