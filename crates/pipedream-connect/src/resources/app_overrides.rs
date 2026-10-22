//! App override administration operations.

use std::sync::Arc;

use async_trait::async_trait;
use pipedream_connect_types::{
    AppOverride, AppOverrideId, CreateAppOverrideRequest, ListAppOverridesRequest, Page,
    UpdateAppOverrideRequest,
};

use crate::client::RequestDispatcher;
use crate::error::Result;

use super::admin_operations;
use super::operation;

/// App override operations, consumed through a shared trait object.
#[cfg_attr(
    any(test, feature = "unimock"),
    unimock::unimock(api = AppOverridesApiMock)
)]
#[async_trait]
pub trait AppOverridesApi: Send + Sync {
    /// Lists app overrides with cursor metadata.
    async fn list(&self, request: ListAppOverridesRequest) -> Result<Page<AppOverride>>;
    /// Creates an app override.
    async fn create(&self, request: CreateAppOverrideRequest) -> Result<AppOverride>;
    /// Retrieves an app override and verifies its identifier.
    async fn get(&self, id: AppOverrideId) -> Result<AppOverride>;
    /// Updates an app override and returns the provider response unchanged.
    async fn update(
        &self,
        id: AppOverrideId,
        request: UpdateAppOverrideRequest,
    ) -> Result<AppOverride>;
    /// Deletes an app override.
    async fn delete(&self, id: AppOverrideId) -> Result<()>;
}

/// An app override handle backed by the shared dispatcher.
#[derive(Clone)]
pub struct AppOverrides {
    dispatcher: Arc<dyn RequestDispatcher + Send + Sync>,
}

impl AppOverrides {
    /// Injects shared dispatch without sending a request.
    pub fn new(dispatcher: Arc<dyn RequestDispatcher + Send + Sync>) -> Self {
        Self { dispatcher }
    }
}

#[async_trait]
impl AppOverridesApi for AppOverrides {
    async fn list(&self, request: ListAppOverridesRequest) -> Result<Page<AppOverride>> {
        operation::json(
            &self.dispatcher,
            admin_operations::list_app_overrides(&request),
        )
        .await
    }

    async fn create(&self, request: CreateAppOverrideRequest) -> Result<AppOverride> {
        operation::json(
            &self.dispatcher,
            admin_operations::create_app_override(&request),
        )
        .await
    }

    async fn get(&self, id: AppOverrideId) -> Result<AppOverride> {
        let value = operation::json(
            &self.dispatcher,
            Ok(admin_operations::get_app_override(&id)),
        )
        .await?;
        admin_operations::check_app_override(value, &id)
    }

    async fn update(
        &self,
        id: AppOverrideId,
        request: UpdateAppOverrideRequest,
    ) -> Result<AppOverride> {
        operation::json(
            &self.dispatcher,
            admin_operations::update_app_override(&id, &request),
        )
        .await
    }

    async fn delete(&self, id: AppOverrideId) -> Result<()> {
        operation::no_content(
            &self.dispatcher,
            Ok(admin_operations::delete_app_override(&id)),
        )
        .await
    }
}

#[cfg(test)]
#[path = "_tests_/app_overrides_tests.rs"]
mod app_overrides_tests;
