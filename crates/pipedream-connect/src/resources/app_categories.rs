//! App category retrieval with checked category identities.

use std::sync::Arc;

use async_trait::async_trait;
use http::Method;
use pipedream_connect_types::{AppCategoryId, GetAppCategoryResponse};

use crate::client::RequestDispatcher;
use crate::error::{Error, Result, WireFailure};
use crate::request::{ApiPath, ApiRequest};

/// App category operations, consumed through a shared trait object.
#[cfg_attr(any(test,feature="unimock"),unimock::unimock(api=AppCategoriesApiMock))]
#[async_trait]
pub trait AppCategoriesApi: Send + Sync {
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
    async fn get(&self, id: AppCategoryId) -> Result<GetAppCategoryResponse> {
        let request = ApiRequest::new(
            Method::GET,
            ApiPath::connect(&["app_categories", id.as_str()]),
        );
        let response: GetAppCategoryResponse = self.dispatcher.dispatch(request).await?.json()?;
        if response.id.as_ref() != Some(&id) {
            return Err(Error::internal(WireFailure::IdentityMismatch));
        }
        Ok(response)
    }
}

#[cfg(test)]
#[path = "_tests_/app_categories_tests.rs"]
mod app_categories_tests;
