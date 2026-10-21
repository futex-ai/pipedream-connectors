//! Custom OAuth client administration operations.

use std::sync::Arc;

use async_trait::async_trait;
use pipedream_connect_types::{
    CreateOauthAppRequest, ListOauthAppsRequest, OauthApp, OauthAppId, Page, UpdateOauthAppRequest,
};

use crate::client::RequestDispatcher;
use crate::error::Result;

use super::admin_operations;
use super::operation;

/// Custom OAuth app operations, consumed through a shared trait object.
#[cfg_attr(
    any(test, feature = "unimock"),
    unimock::unimock(api = OauthAppsApiMock)
)]
#[async_trait]
pub trait OauthAppsApi: Send + Sync {
    /// Lists custom OAuth clients with cursor metadata.
    async fn list(&self, request: ListOauthAppsRequest) -> Result<Page<OauthApp>>;
    /// Creates a custom OAuth client.
    async fn create(&self, request: CreateOauthAppRequest) -> Result<OauthApp>;
    /// Retrieves a custom OAuth client and verifies its identifier.
    async fn get(&self, id: OauthAppId) -> Result<OauthApp>;
    /// Updates a custom OAuth client and verifies its identifier.
    async fn update(&self, id: OauthAppId, request: UpdateOauthAppRequest) -> Result<OauthApp>;
    /// Deletes a custom OAuth client.
    async fn delete(&self, id: OauthAppId) -> Result<()>;
}

/// A custom OAuth app handle backed by the shared dispatcher.
#[derive(Clone)]
pub struct OauthApps {
    dispatcher: Arc<dyn RequestDispatcher + Send + Sync>,
}

impl OauthApps {
    /// Injects shared dispatch without sending a request.
    pub fn new(dispatcher: Arc<dyn RequestDispatcher + Send + Sync>) -> Self {
        Self { dispatcher }
    }
}

#[async_trait]
impl OauthAppsApi for OauthApps {
    async fn list(&self, request: ListOauthAppsRequest) -> Result<Page<OauthApp>> {
        operation::json(
            &self.dispatcher,
            admin_operations::list_oauth_apps(&request),
        )
        .await
    }

    async fn create(&self, request: CreateOauthAppRequest) -> Result<OauthApp> {
        operation::json(
            &self.dispatcher,
            admin_operations::create_oauth_app(&request),
        )
        .await
    }

    async fn get(&self, id: OauthAppId) -> Result<OauthApp> {
        let app =
            operation::json(&self.dispatcher, Ok(admin_operations::get_oauth_app(&id))).await?;
        admin_operations::check_oauth_app(app, &id)
    }

    async fn update(&self, id: OauthAppId, request: UpdateOauthAppRequest) -> Result<OauthApp> {
        let app = operation::json(
            &self.dispatcher,
            admin_operations::update_oauth_app(&id, &request),
        )
        .await?;
        admin_operations::check_oauth_app(app, &id)
    }

    async fn delete(&self, id: OauthAppId) -> Result<()> {
        operation::dispatch(
            &self.dispatcher,
            Ok(admin_operations::delete_oauth_app(&id)),
        )
        .await
        .map(|_| ())
    }
}

#[cfg(test)]
#[path = "_tests_/oauth_apps_tests.rs"]
mod oauth_apps_tests;
