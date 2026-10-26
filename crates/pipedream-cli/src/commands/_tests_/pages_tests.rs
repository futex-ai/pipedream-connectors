//! Tests for cursor flag validation and the mapping of traversal stop reasons.

use std::num::NonZeroUsize;

use clap::Parser;
use pipedream_connect::pagination::{Pagination, PaginationEnd};
use pipedream_connect_types::{Cursor, IdentifierKind, PageRequest};

use crate::argument::Argument;
use crate::commands::pages::{End, PageArgs, Paging, traversal};
use crate::document::Notice;
use crate::error::Error;

/// A parser for the page flags alone.
#[derive(Debug, Parser)]
struct Flags {
    #[command(flatten)]
    pages: PageArgs,
}

fn paging(args: &[&str]) -> crate::error::Result<Paging> {
    let mut argv = vec!["flags"];
    argv.extend_from_slice(args);
    Flags::try_parse_from(argv).unwrap().pages.paging()
}

fn budget(pages: usize) -> Option<NonZeroUsize> {
    NonZeroUsize::new(pages)
}

#[test]
fn absent_flags_request_one_default_page() {
    assert_eq!(
        paging(&[]).unwrap(),
        Paging {
            page: PageRequest::new(),
            budget: None,
        }
    );
}

#[test]
fn flags_build_the_first_page_and_the_budget() {
    let paging = paging(&["--limit", "5", "--before", "cursor_0", "--all"]).unwrap();
    assert_eq!(paging.page.limit, 5);
    assert_eq!(paging.page.before, Some(Cursor::new("cursor_0").unwrap()));
    assert_eq!(paging.page.after, None);
    assert_eq!(paging.budget, budget(10));
    let paging = self::paging(&["--all", "--max-pages", "3"]).unwrap();
    assert_eq!(paging.budget, budget(3));
}

#[test]
fn invalid_cursors_name_their_flag() {
    let error = paging(&["--after", " "]).unwrap_err();
    assert!(matches!(
        error,
        Error::InvalidArgument {
            argument: Argument::After,
            cause: pipedream_connect_types::Error::InvalidIdentifier {
                kind: IdentifierKind::Cursor
            },
        }
    ));
}

#[test]
fn the_budget_needs_all_and_cursors_take_one_direction() {
    for args in [
        &["--max-pages", "2"][..],
        &["--after", "a", "--before", "b"],
    ] {
        let mut argv = vec!["flags"];
        argv.extend_from_slice(args);
        assert!(Flags::try_parse_from(argv).is_err(), "{args:?}");
    }
}

#[test]
fn page_sizes_outside_the_sdk_range_name_the_range() {
    for limit in ["0", "201", "70000", "-1", "ten"] {
        let error = paging(&["--limit", limit]).unwrap_err();
        assert!(
            matches!(
                error,
                Error::InvalidArgument {
                    argument: Argument::Limit,
                    cause: pipedream_connect_types::Error::OutOfRange {
                        minimum: 1,
                        maximum: 200,
                        ..
                    },
                }
            ),
            "{limit}: {error:?}"
        );
    }
}

#[test]
fn budgets_must_be_positive_whole_numbers_without_an_upper_limit() {
    for budget in ["0", "-1", "many"] {
        let error = paging(&["--all", "--max-pages", budget]).unwrap_err();
        assert!(
            matches!(
                error,
                Error::InvalidCount {
                    argument: Argument::MaxPages
                }
            ),
            "{budget}: {error:?}"
        );
    }
    let paging = paging(&["--all", "--max-pages", "1000000"]).unwrap();
    assert_eq!(paging.budget, budget(1_000_000));
}

fn pagination(end: PaginationEnd, next: Option<PageRequest>) -> Pagination<u8> {
    Pagination {
        data: vec![1, 2],
        pages: 2,
        next,
        end,
    }
}

#[test]
fn traversal_maps_each_stop_reason_to_its_notice() {
    let next = PageRequest {
        after: Some(Cursor::new("cursor_2").unwrap()),
        ..PageRequest::default()
    };
    let (exhausted, notices) = traversal(pagination(PaginationEnd::Exhausted, None));
    assert_eq!((exhausted.end, exhausted.pages), (End::Exhausted, 2));
    assert_eq!(exhausted.data, [1, 2]);
    assert!(notices.is_empty());

    let (limited, notices) = traversal(pagination(PaginationEnd::PageLimit, Some(next.clone())));
    assert_eq!(limited.end, End::PageLimit);
    assert_eq!(limited.next, Some(next.clone()));
    assert_eq!(notices, [Notice::PageLimit { pages: 2, next }]);

    let (stuck, notices) = traversal(pagination(PaginationEnd::StuckCursor, None));
    assert_eq!(stuck.end, End::StuckCursor);
    assert_eq!(notices, [Notice::StuckCursor]);
}
