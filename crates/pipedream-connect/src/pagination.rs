//! Bounded cursor traversal with an injectable page source.

use std::num::NonZeroUsize;
use std::sync::Arc;

use async_trait::async_trait;
use pipedream_connect_types::{Error, Page, PageRequest};

use crate::error::{Result, validate};

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

/// Bounded traversal consumed through a shared trait object.
#[cfg_attr(any(test, feature = "unimock"), unimock::unimock(api = PaginatorMock))]
#[async_trait]
pub trait Paginator<T: Send + Sync + 'static>: Send + Sync {
    /// Fetches at most `page_limit` pages without repeating a stuck cursor.
    async fn paginate(
        &self,
        initial: PageRequest,
        page_limit: NonZeroUsize,
    ) -> Result<Pagination<T>>;
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
    ) -> Result<Pagination<T>> {
        validate(initial.validate())?;
        let mut request = initial;
        let mut result = Pagination {
            data: Vec::new(),
            pages: 0,
            next: None,
            end: PaginationEnd::PageLimit,
        };
        for _ in 0..page_limit.get() {
            let page = self.source.fetch(request.clone()).await?;
            result.pages += 1;
            let empty = page.data.is_empty();
            result.data.extend(page.data);
            let next = if empty {
                Ok(None)
            } else {
                request.next(&page.page_info)
            };
            let next = match next {
                Err(Error::StuckCursor) => {
                    tracing::warn!("Pipedream pagination stopped on a repeated cursor");
                    result.end = PaginationEnd::StuckCursor;
                    return Ok(result);
                }
                next => validate(next)?,
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
