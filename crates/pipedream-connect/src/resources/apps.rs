//! Public app discovery with cursor filters and checked app identities.

use std::sync::Arc;

use async_trait::async_trait;
use http::Method;
use pipedream_connect_types::{App, AppSlug, GetAppResponse, ListAppsRequest, Page};

use crate::client::RequestDispatcher;
use crate::error::{Error, Result, WireFailure};
use crate::request::{ApiPath, ApiRequest};

use super::queries;

/// App catalog operations, consumed through a shared trait object.
#[cfg_attr(any(test,feature="unimock"),unimock::unimock(api=AppsApiMock))]
#[async_trait]
pub trait AppsApi: Send + Sync {
    /// Lists apps with every documented filter and provider cursor metadata.
    async fn list(&self, request: ListAppsRequest) -> Result<Page<App>>;
    /// Retrieves an app by slug and verifies its returned slug.
    async fn get(&self, slug: AppSlug) -> Result<GetAppResponse>;
}

/// An app handle backed by the shared authenticated dispatcher.
#[derive(Clone)]
pub struct Apps {
    dispatcher: Arc<dyn RequestDispatcher + Send + Sync>,
}

impl Apps {
    /// Injects shared dispatch without sending a request.
    pub fn new(dispatcher: Arc<dyn RequestDispatcher + Send + Sync>) -> Self {
        Self { dispatcher }
    }
}

#[async_trait]
impl AppsApi for Apps {
    async fn list(&self, request: ListAppsRequest) -> Result<Page<App>> {
        self.dispatcher
            .dispatch(queries::apps(request)?)
            .await?
            .json()
    }

    async fn get(&self, slug: AppSlug) -> Result<GetAppResponse> {
        let request = ApiRequest::new(Method::GET, ApiPath::connect(&["apps", slug.as_str()]));
        let response: GetAppResponse = self.dispatcher.dispatch(request).await?.json()?;
        if response.data.name_slug != slug {
            return Err(Error::internal(WireFailure::IdentityMismatch));
        }
        Ok(response)
    }
}

#[cfg(test)]
#[path = "_tests_/apps_tests.rs"]
mod apps_tests;
