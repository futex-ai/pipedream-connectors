//! Typed actions operations through the shared authenticated dispatcher.

use std::sync::Arc;

use async_trait::async_trait;
use pipedream_connect_types::{
    Component, ComponentKey, ConfigurePropRequest, ConfigurePropResponse, GetComponentRequest,
    GetComponentResponse, ListRegistryRequest, Page, ReloadPropsRequest, ReloadPropsResponse,
    RunActionRequest, RunActionResponse,
};

use crate::client::RequestDispatcher;
use crate::error::Result;

use super::catalog::{CatalogGroup, check_component, get_request};
use super::{execution, queries};

/// Registry actions operations, consumed through a shared trait object.
#[cfg_attr(any(test, feature = "unimock"), unimock::unimock(api = ActionsApiMock))]
#[async_trait]
pub trait ActionsApi: Send + Sync {
    /// Lists registry actions with their documented filters and cursor metadata.
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
    /// Runs an action once, with an explicit optional blocking flag and stash selector.
    async fn run(&self, request: RunActionRequest) -> Result<RunActionResponse>;
}

/// A registry actions handle backed by the shared authenticated dispatcher.
#[derive(Clone)]
pub struct Actions {
    dispatcher: Arc<dyn RequestDispatcher + Send + Sync>,
}

impl Actions {
    /// Injects shared dispatch without sending a request.
    pub fn new(dispatcher: Arc<dyn RequestDispatcher + Send + Sync>) -> Self {
        Self { dispatcher }
    }
}

#[async_trait]
impl ActionsApi for Actions {
    async fn list(&self, request: ListRegistryRequest) -> Result<Page<Component>> {
        self.dispatcher
            .dispatch(queries::registry(CatalogGroup::Actions, request)?)
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
            .dispatch(get_request(CatalogGroup::Actions, &key, request))
            .await?
            .json()?;
        check_component(response, &key)
    }

    async fn configure(&self, request: ConfigurePropRequest) -> Result<ConfigurePropResponse> {
        self.dispatcher
            .dispatch(execution::configure(CatalogGroup::Actions, &request)?)
            .await?
            .json()
    }

    async fn reload_props(&self, request: ReloadPropsRequest) -> Result<ReloadPropsResponse> {
        self.dispatcher
            .dispatch(execution::reload(CatalogGroup::Actions, &request)?)
            .await?
            .json()
    }

    async fn run(&self, request: RunActionRequest) -> Result<RunActionResponse> {
        self.dispatcher
            .dispatch(execution::run(&request)?)
            .await?
            .json()
    }
}

#[cfg(test)]
#[path = "_tests_/actions/mod.rs"]
mod actions_tests;
