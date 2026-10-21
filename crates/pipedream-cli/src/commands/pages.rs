//! Cursor flags, and single-page or bounded multi-page listing.

use std::num::NonZeroUsize;
use std::sync::Arc;

use clap::Args;
use pipedream_connect::pagination::{
    BoundedPaginator, PageSource, Pagination, PaginationEnd, PaginationError, Paginator,
};
use pipedream_connect_types::{Cursor, PageRequest};
use serde::Serialize;

use crate::argument::Argument;
use crate::document::{Document, Notice, Output, Secrets};
use crate::error::{Error, Interruption, Result, called, checked};
use crate::redact::Redact;
use crate::target::Target;

/// The cursor flags of every paged list command.
#[derive(Debug, Args)]
pub(crate) struct PageArgs {
    /// Records for each page, from 1 to 200. The default is 100.
    #[arg(long, value_name = "COUNT", help_heading = "Pages")]
    limit: Option<u16>,
    /// Read the records after this cursor.
    #[arg(
        long,
        value_name = "CURSOR",
        conflicts_with = "before",
        help_heading = "Pages"
    )]
    after: Option<String>,
    /// Read the records before this cursor.
    #[arg(long, value_name = "CURSOR", help_heading = "Pages")]
    before: Option<String>,
    /// Read every page, up to the --max-pages budget.
    #[arg(long, help_heading = "Pages")]
    all: bool,
    /// The largest number of pages that --all reads.
    #[arg(
        long,
        value_name = "COUNT",
        default_value = "10",
        requires = "all",
        help_heading = "Pages"
    )]
    max_pages: NonZeroUsize,
}

/// A validated first page and the page budget of `--all`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Paging {
    /// The first page request.
    pub(crate) page: PageRequest,
    /// The page budget, when the caller passed `--all`.
    pub(crate) budget: Option<NonZeroUsize>,
}

impl PageArgs {
    /// Validates the cursor flags. Sends no request.
    pub(crate) fn paging(self) -> Result<Paging> {
        let mut page = match self.limit {
            Some(limit) => checked(PageRequest::with_limit(limit), Argument::Limit)?,
            None => PageRequest::new(),
        };
        page.after = self
            .after
            .map(|cursor| checked(Cursor::new(cursor), Argument::After))
            .transpose()?;
        page.before = self
            .before
            .map(|cursor| checked(Cursor::new(cursor), Argument::Before))
            .transpose()?;
        Ok(Paging {
            page,
            budget: self.all.then_some(self.max_pages),
        })
    }
}

/// Why a multi-page listing stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum End {
    /// The listing read the last page.
    Exhausted,
    /// The listing reached its page budget. `next` continues it.
    PageLimit,
    /// Pipedream repeated a page cursor.
    StuckCursor,
}

/// The records of a multi-page listing, with its page count and stop reason.
#[derive(Debug, Serialize)]
pub(crate) struct Traversal<T> {
    /// The records in provider order.
    pub(crate) data: Vec<T>,
    /// The number of pages read.
    pub(crate) pages: usize,
    /// Why the listing stopped.
    pub(crate) end: End,
    /// The request that continues the listing after the page budget.
    pub(crate) next: Option<PageRequest>,
}

impl<T: Redact> Redact for Traversal<T> {
    fn redact(&mut self) {
        self.data.redact();
    }
}

/// Lists one page, or every page within the `--all` budget, as one document.
pub(crate) async fn list<T>(
    source: Arc<dyn PageSource<T> + Send + Sync>,
    paging: Paging,
    target: Target,
    secrets: Secrets,
) -> Result<Output>
where
    T: Serialize + Redact + Send + Sync + 'static,
{
    let Some(budget) = paging.budget else {
        let page = called(source.fetch(paging.page).await, target)?;
        return Ok(Document::redacted(page, secrets)?.into());
    };
    let pagination = match BoundedPaginator::new(source)
        .paginate(paging.page, budget)
        .await
    {
        Ok(pagination) => pagination,
        Err(PaginationError::Interrupted {
            pages,
            request,
            source,
            ..
        }) => {
            return Err(Error::Interrupted(Box::new(Interruption {
                target,
                pages,
                request,
                cause: source,
            })));
        }
    };
    let (traversal, notices) = traversal(pagination);
    Ok(Output {
        document: Document::redacted(traversal, secrets)?,
        notices,
    })
}

fn traversal<T>(pagination: Pagination<T>) -> (Traversal<T>, Vec<Notice>) {
    let (end, notices) = match (&pagination.end, &pagination.next) {
        (PaginationEnd::Exhausted, _) => (End::Exhausted, Vec::new()),
        (PaginationEnd::PageLimit, Some(next)) => (
            End::PageLimit,
            vec![Notice::PageLimit {
                pages: pagination.pages,
                next: next.clone(),
            }],
        ),
        (PaginationEnd::PageLimit, None) => (End::PageLimit, Vec::new()),
        (PaginationEnd::StuckCursor, _) => (End::StuckCursor, vec![Notice::StuckCursor]),
    };
    let traversal = Traversal {
        data: pagination.data,
        pages: pagination.pages,
        end,
        next: pagination.next,
    };
    (traversal, notices)
}

#[cfg(test)]
#[path = "_tests_/pages_tests.rs"]
mod pages_tests;
