//! Typed components operations through the shared authenticated dispatcher.

use std::sync::Arc;

use async_trait::async_trait;
use pipedream_connect_types::{
    Component, ComponentKey, ConfigurePropRequest, ConfigurePropResponse, GetComponentRequest,
    GetComponentResponse, ListComponentsRequest, Page, ReloadPropsRequest, ReloadPropsResponse,
};

use crate::client::RequestDispatcher;
use crate::error::Result;

use super::operation::{self, CatalogGroup};

/// Registry components operations, consumed through a shared trait object.
#[cfg_attr(
    any(test, feature = "unimock"),
    unimock::unimock(api = ComponentsApiMock)
)]
#[async_trait]
pub trait ComponentsApi: Send + Sync {
    /// Lists registry components with their documented filters and cursor metadata.
    async fn list(&self, request: ListComponentsRequest) -> Result<Page<Component>>;
    /// Retrieves a versioned registry entry and verifies its component key.
    async fn get(
        &self,
        key: ComponentKey,
        request: GetComponentRequest,
    ) -> Result<GetComponentResponse>;
    /// Configures remote property options with the current external-user context.
    async fn configure(&self, request: ConfigurePropRequest) -> Result<ConfigurePropResponse>;
    /// Reloads dynamic properties without dropping unknown property kinds.
    async fn reload_props(&self, request: ReloadPropsRequest) -> Result<ReloadPropsResponse>;
}

/// A registry components handle backed by the shared authenticated dispatcher.
#[derive(Clone)]
pub struct Components {
    dispatcher: Arc<dyn RequestDispatcher + Send + Sync>,
}

impl Components {
    /// Injects shared dispatch without sending a request.
    pub fn new(dispatcher: Arc<dyn RequestDispatcher + Send + Sync>) -> Self {
        Self { dispatcher }
    }
}

#[async_trait]
impl ComponentsApi for Components {
    async fn list(&self, request: ListComponentsRequest) -> Result<Page<Component>> {
        operation::json(&self.dispatcher, operation::list_components(&request)).await
    }

    async fn get(
        &self,
        key: ComponentKey,
        request: GetComponentRequest,
    ) -> Result<GetComponentResponse> {
        let response = operation::json(
            &self.dispatcher,
            operation::get_component(CatalogGroup::Components, &key, &request),
        )
        .await?;
        operation::check_component(response, &key)
    }

    async fn configure(&self, request: ConfigurePropRequest) -> Result<ConfigurePropResponse> {
        operation::json(
            &self.dispatcher,
            operation::configure(CatalogGroup::Components, &request),
        )
        .await
    }

    async fn reload_props(&self, request: ReloadPropsRequest) -> Result<ReloadPropsResponse> {
        operation::json(
            &self.dispatcher,
            operation::reload(CatalogGroup::Components, &request),
        )
        .await
    }
}

#[cfg(test)]
#[path = "_tests_/components_tests.rs"]
mod components_tests;
