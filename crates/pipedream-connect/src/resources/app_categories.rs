//! App category retrieval with checked category identities.

use std::sync::Arc;

use async_trait::async_trait;
use pipedream_connect_types::{AppCategoryId, GetAppCategoryResponse, ListAppCategoriesResponse};

use crate::client::RequestDispatcher;
use crate::error::Result;

use super::operation;

/// App category operations, consumed through a shared trait object.
#[cfg_attr(
    any(test, feature = "unimock"),
    unimock::unimock(api = AppCategoriesApiMock)
)]
#[async_trait]
pub trait AppCategoriesApi: Send + Sync {
    /// Lists every app category without pagination or query parameters.
    async fn list(&self) -> Result<ListAppCategoriesResponse>;
    /// Retrieves one category and verifies its returned identifier.
    async fn get(&self, id: AppCategoryId) -> Result<GetAppCategoryResponse>;
}

/// A category handle backed by the shared authenticated dispatcher.
#[derive(Clone)]
pub struct AppCategories {
    dispatcher: Arc<dyn RequestDispatcher + Send + Sync>,
}

impl AppCategories {
    /// Injects shared dispatch without sending a request.
    pub fn new(dispatcher: Arc<dyn RequestDispatcher + Send + Sync>) -> Self {
        Self { dispatcher }
    }
}

#[async_trait]
impl AppCategoriesApi for AppCategories {
    async fn list(&self) -> Result<ListAppCategoriesResponse> {
        operation::json(&self.dispatcher, Ok(operation::list_categories())).await
    }

    async fn get(&self, id: AppCategoryId) -> Result<GetAppCategoryResponse> {
        let response = operation::json(&self.dispatcher, Ok(operation::get_category(&id))).await?;
        operation::check_category(response, &id)
    }
}

#[cfg(test)]
#[path = "_tests_/app_categories_tests.rs"]
mod app_categories_tests;
