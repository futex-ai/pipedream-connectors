//! Typed triggers operations through the shared authenticated dispatcher.

use std::sync::Arc;

use async_trait::async_trait;
use pipedream_connect_types::{
    Component, ComponentKey, ConfigurePropRequest, ConfigurePropResponse, GetComponentRequest,
    GetComponentResponse, ListRegistryRequest, Page, ReloadPropsRequest, ReloadPropsResponse,
};

use crate::client::RequestDispatcher;
use crate::error::Result;

use super::catalog::{CatalogGroup, check_component, get_request};
use super::{execution, queries};

/// Registry triggers operations, consumed through a shared trait object.
#[cfg_attr(any(test, feature = "unimock"), unimock::unimock(api = TriggersApiMock))]
#[async_trait]
pub trait TriggersApi: Send + Sync {
    /// Lists registry triggers with their documented filters and cursor metadata.
    async fn list(&self, request: ListRegistryRequest) -> Result<Page<Component>>;
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

/// A registry triggers handle backed by the shared authenticated dispatcher.
#[derive(Clone)]
pub struct Triggers {
    dispatcher: Arc<dyn RequestDispatcher + Send + Sync>,
}

impl Triggers {
    /// Injects shared dispatch without sending a request.
    pub fn new(dispatcher: Arc<dyn RequestDispatcher + Send + Sync>) -> Self {
        Self { dispatcher }
    }
}

#[async_trait]
impl TriggersApi for Triggers {
    async fn list(&self, request: ListRegistryRequest) -> Result<Page<Component>> {
        self.dispatcher
            .dispatch(queries::registry(CatalogGroup::Triggers, request)?)
            .await?
            .json()
    }

    async fn get(
        &self,
        key: ComponentKey,
        request: GetComponentRequest,
    ) -> Result<GetComponentResponse> {
        let response = self
            .dispatcher
            .dispatch(get_request(CatalogGroup::Triggers, &key, request))
            .await?
            .json()?;
        check_component(response, &key)
    }

    async fn configure(&self, request: ConfigurePropRequest) -> Result<ConfigurePropResponse> {
        self.dispatcher
            .dispatch(execution::configure(CatalogGroup::Triggers, &request)?)
            .await?
            .json()
    }

    async fn reload_props(&self, request: ReloadPropsRequest) -> Result<ReloadPropsResponse> {
        self.dispatcher
            .dispatch(execution::reload(CatalogGroup::Triggers, &request)?)
            .await?
            .json()
    }
}

#[cfg(test)]
#[path = "_tests_/triggers_tests.rs"]
mod triggers_tests;
