//! Tests for exit codes, interrupted listings, and internal failures.

use std::io;

use pipedream_connect_types::{Cursor, PageRequest};

use crate::error::{Error, Interruption};
use crate::report::exit::Exit;
use crate::report::failure::{Failure, count, describe, resume};
use crate::target::Target;

#[test]
fn exit_codes_match_the_documented_set() {
    let codes = [
        (Exit::Success, 0),
        (Exit::Internal, 1),
        (Exit::Usage, 2),
        (Exit::Authentication, 3),
        (Exit::NotFound, 4),
        (Exit::RateLimited, 5),
        (Exit::Unavailable, 6),
        (Exit::Rejected, 7),
    ];
    for (exit, code) in codes {
        assert_eq!(exit.code(), code, "{exit:?}");
    }
}

#[test]
fn interrupted_listings_name_the_resume_cursor() {
    let request = PageRequest {
        after: Some(Cursor::new("cursor_2").unwrap()),
        ..PageRequest::default()
    };
    let interrupted = |pages, request| {
        describe(&Error::Interrupted(Box::new(Interruption {
            target: Target::Actions,
            pages,
            request,
            cause: pipedream_connect::error::Error::Unavailable { status: None },
        })))
    };
    assert_eq!(
        interrupted(1, request.clone()),
        Failure::new(
            Exit::Unavailable,
            "Pipedream did not respond. Try again later. The CLI read 1 page before the \
             failure. To continue, run the command again with --after cursor_2."
        )
    );
    assert_eq!(
        interrupted(0, PageRequest::default()),
        Failure::new(
            Exit::Unavailable,
            "Pipedream did not respond. Try again later."
        )
    );
}

#[test]
fn resume_hints_follow_the_cursor_direction() {
    let before = PageRequest {
        before: Some(Cursor::new("cursor_0").unwrap()),
        ..PageRequest::default()
    };
    assert_eq!(
        resume(&before).as_deref(),
        Some("To continue, run the command again with --before cursor_0.")
    );
    assert_eq!(resume(&PageRequest::default()), None);
    assert_eq!(count(1), "1 page");
    assert_eq!(count(4), "4 pages");
}

#[test]
fn internal_failures_use_one_generic_message() {
    let error = Error::internal(io::Error::other("encoder detail"));
    let failure = describe(&error);
    assert_eq!(failure.exit, Exit::Internal);
    assert_eq!(
        failure.message,
        "The CLI could not complete the command. Report this problem with the command that \
         you ran."
    );
}
