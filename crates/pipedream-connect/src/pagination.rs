//! Bounded cursor traversal with an injectable page source.

use std::fmt;
use std::future::Future;
use std::num::NonZeroUsize;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use pipedream_connect_types::{self, Page, PageRequest};
use thiserror::Error;

use crate::error::{self, Result, validate};

/// A paged list request whose filters can be reused for another page.
pub trait PagedRequest: Clone + Send + Sync + 'static {
    /// Replaces the cursor page while retaining every other filter.
    fn set_page(&mut self, page: PageRequest);
}

impl PagedRequest for pipedream_connect_types::ListAppsRequest {
    fn set_page(&mut self, page: PageRequest) {
        self.page = page;
    }
}

impl PagedRequest for pipedream_connect_types::ListComponentsRequest {
    fn set_page(&mut self, page: PageRequest) {
        self.page = page;
    }
}

impl PagedRequest for pipedream_connect_types::ListRegistryRequest {
    fn set_page(&mut self, page: PageRequest) {
        self.page = page;
    }
}

impl PagedRequest for pipedream_connect_types::ListDeployedTriggersRequest {
    fn set_page(&mut self, page: PageRequest) {
        self.page = page;
    }
}

impl PagedRequest for pipedream_connect_types::ListAccountsRequest {
    fn set_page(&mut self, page: PageRequest) {
        self.page = page;
    }
}

impl PagedRequest for pipedream_connect_types::ListProjectsRequest {
    fn set_page(&mut self, page: PageRequest) {
        self.page = page;
    }
}

impl PagedRequest for pipedream_connect_types::ListOauthAppsRequest {
    fn set_page(&mut self, page: PageRequest) {
        self.page = page;
    }
}

impl PagedRequest for pipedream_connect_types::ListAppOverridesRequest {
    fn set_page(&mut self, page: PageRequest) {
        self.page = page;
    }
}

impl PagedRequest for pipedream_connect_types::ListUsersRequest {
    fn set_page(&mut self, page: PageRequest) {
        self.page = page;
    }
}

type PageFetch<T, R> =
    dyn Fn(R) -> Pin<Box<dyn Future<Output = Result<Page<T>>> + Send>> + Send + Sync;

/// Adapts a filtered resource list into the generic bounded paginator.
pub struct FilteredPageSource<T, R: PagedRequest> {
    filters: R,
    fetch: Box<PageFetch<T, R>>,
}

impl<T, R: PagedRequest> FilteredPageSource<T, R> {
    /// Keeps `filters` fixed and invokes `fetch` with only its page changed.
    pub fn new<F, Fut>(filters: R, fetch: F) -> Self
    where
        F: Fn(R) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Page<T>>> + Send + 'static,
    {
        Self {
            filters,
            fetch: Box::new(move |request| Box::pin(fetch(request))),
        }
    }
}

#[async_trait]
impl<T: Send + Sync + 'static, R: PagedRequest> PageSource<T> for FilteredPageSource<T, R> {
    async fn fetch(&self, page: PageRequest) -> Result<Page<T>> {
        let mut request = self.filters.clone();
        request.set_page(page);
        (self.fetch)(request).await
    }
}

/// One resource's page-fetching boundary.
#[cfg_attr(any(test, feature = "unimock"), unimock::unimock(api = PageSourceMock))]
#[async_trait]
pub trait PageSource<T: Send + Sync + 'static>: Send + Sync {
    /// Fetches a validated cursor page through the resource API.
    async fn fetch(&self, request: PageRequest) -> Result<Page<T>>;
}

/// Why bounded traversal stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaginationEnd {
    /// No further page exists.
    Exhausted,
    /// The configured page count was reached. `next` can resume traversal.
    PageLimit,
    /// The provider repeated a cursor. No further request was sent.
    StuckCursor,
}

/// Records and resumption state from a bounded traversal.
#[derive(Debug)]
pub struct Pagination<T> {
    /// Records in page order.
    pub data: Vec<T>,
    /// Number of completed page requests.
    pub pages: usize,
    /// Next request when the page limit stopped traversal.
    pub next: Option<PageRequest>,
    /// Reason traversal stopped.
    pub end: PaginationEnd,
}

/// A failed traversal that retains completed records and the failed request.
#[derive(Error)]
pub enum PaginationError<T> {
    /// Validation or a page request failed. The host can inspect or resume it.
    #[error("[pipedream-connect/pagination] interrupted after {pages} pages: {source}")]
    Interrupted {
        /// Records from all completed pages, in provider order.
        data: Vec<T>,
        /// Number of completed page requests.
        pages: usize,
        /// Request that failed, or the invalid initial request.
        request: PageRequest,
        /// Original typed client error.
        #[source]
        source: error::Error,
    },
}

impl<T> fmt::Debug for PaginationError<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Interrupted {
                data,
                pages,
                request,
                source,
            } => formatter
                .debug_struct("Interrupted")
                .field("records", &data.len())
                .field("pages", pages)
                .field("request", request)
                .field("source", source)
                .finish(),
        }
    }
}

/// Traversal result whose error retains partial records and resumption state.
pub type PaginationResult<T> = std::result::Result<Pagination<T>, PaginationError<T>>;

/// Bounded traversal consumed through a shared trait object.
#[cfg_attr(any(test, feature = "unimock"), unimock::unimock(api = PaginatorMock))]
#[async_trait]
pub trait Paginator<T: Send + Sync + 'static>: Send + Sync {
    /// Fetches at most `page_limit` pages without repeating a stuck cursor.
    async fn paginate(&self, initial: PageRequest, page_limit: NonZeroUsize)
    -> PaginationResult<T>;
}

/// Composition root for a resource's bounded pagination helper.
pub struct BoundedPaginator<T: Send + Sync + 'static> {
    source: Arc<dyn PageSource<T> + Send + Sync>,
}

impl<T: Send + Sync + 'static> BoundedPaginator<T> {
    /// Injects the page source without fetching data.
    pub fn new(source: Arc<dyn PageSource<T> + Send + Sync>) -> Self {
        Self { source }
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> Paginator<T> for BoundedPaginator<T> {
    async fn paginate(
        &self,
        initial: PageRequest,
        page_limit: NonZeroUsize,
    ) -> PaginationResult<T> {
        if let Err(source) = validate(initial.validate()) {
            return Err(PaginationError::Interrupted {
                data: Vec::new(),
                pages: 0,
                request: initial,
                source,
            });
        }
        let mut request = initial;
        let mut result = Pagination {
            data: Vec::new(),
            pages: 0,
            next: None,
            end: PaginationEnd::PageLimit,
        };
        for _ in 0..page_limit.get() {
            let page = match self.source.fetch(request.clone()).await {
                Ok(page) => page,
                Err(source) => {
                    return Err(PaginationError::Interrupted {
                        data: result.data,
                        pages: result.pages,
                        request,
                        source,
                    });
                }
            };
            result.pages += 1;
            let empty = page.data.is_empty();
            result.data.extend(page.data);
            let next = if empty {
                Ok(None)
            } else {
                request.next(&page.page_info)
            };
            let next = match next {
                Err(pipedream_connect_types::Error::StuckCursor) => {
                    tracing::warn!("Pipedream pagination stopped on a repeated cursor");
                    result.end = PaginationEnd::StuckCursor;
                    return Ok(result);
                }
                next => match validate(next) {
                    Ok(next) => next,
                    Err(source) => {
                        return Err(PaginationError::Interrupted {
                            data: result.data,
                            pages: result.pages,
                            request,
                            source,
                        });
                    }
                },
            };
            let Some(next) = next else {
                result.end = PaginationEnd::Exhausted;
                return Ok(result);
            };
            request = next;
        }
        result.next = Some(request);
        Ok(result)
    }
}
