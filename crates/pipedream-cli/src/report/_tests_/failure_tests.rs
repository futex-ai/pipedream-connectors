//! Tests for exit codes, interrupted listings, and internal failures.

use std::io;
use std::num::NonZeroU64;
use std::time::Duration;

use pipedream_connect_types::{Cursor, PageRequest};

use crate::error::{Error, Interruption};
use crate::report::exit::Exit;
use crate::report::failure::{Failure, count, describe, resume};
use crate::settings::DEFAULT_ORIGIN;
use crate::target::Target;

fn after(cursor: &str) -> PageRequest {
    PageRequest {
        after: Some(Cursor::new(cursor).unwrap()),
        ..PageRequest::default()
    }
}

fn interrupted(
    pages: usize,
    request: PageRequest,
    cause: pipedream_connect::error::Error,
) -> Failure {
    let interruption = Interruption {
        target: Target::Actions,
        pages,
        request,
        cause,
    };
    describe(&Error::Interrupted(Box::new(interruption)), DEFAULT_ORIGIN)
}

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
fn a_retryable_interruption_gives_the_resume_command_as_its_one_action() {
    let unavailable = pipedream_connect::error::Error::Unavailable {
        status: None,
        retry_after: None,
    };
    assert_eq!(
        interrupted(1, after("cursor_2"), unavailable),
        Failure::new(
            Exit::Unavailable,
            "The CLI could not reach Pipedream at https://api.pipedream.com. The CLI read 1 page \
             before the failure. To continue, run the command again with --after 'cursor_2'."
        )
    );
    let limited = pipedream_connect::error::Error::RateLimited {
        retry_after: Some(Duration::from_secs(30)),
        reset_after: None,
        limit: None,
        remaining: None,
        reset: None,
    };
    assert_eq!(
        interrupted(2, after("cursor_3"), limited),
        Failure::new(
            Exit::RateLimited,
            "Pipedream rate limited the request. The CLI read 2 pages before the failure. Wait \
             30 seconds, then run the command again with --after 'cursor_3'."
        )
    );
}

#[test]
fn a_server_delay_comes_before_the_resume_hint() {
    let unavailable = pipedream_connect::error::Error::Unavailable {
        status: Some(503),
        retry_after: Some(Duration::from_secs(30)),
    };
    assert_eq!(
        interrupted(1, after("cursor_5"), unavailable),
        Failure::new(
            Exit::Unavailable,
            "Pipedream at https://api.pipedream.com is unavailable with status 503. The CLI \
             read 1 page before the failure. Wait 30 seconds, then run the command again with \
             --after 'cursor_5'."
        )
    );
}

#[test]
fn a_zero_wait_never_reaches_the_resume_hint() {
    let limited = pipedream_connect::error::Error::RateLimited {
        retry_after: Some(Duration::ZERO),
        reset_after: None,
        limit: None,
        remaining: None,
        reset: None,
    };
    assert_eq!(
        interrupted(2, after("cursor_4"), limited),
        Failure::new(
            Exit::RateLimited,
            "Pipedream rate limited the request. The CLI read 2 pages before the failure. To \
             continue, run the command again with --after 'cursor_4'."
        )
    );
}

#[test]
fn other_interruptions_keep_their_own_action() {
    let forbidden = pipedream_connect::error::Error::Forbidden { message: None };
    assert_eq!(
        interrupted(2, after("cursor_3"), forbidden),
        Failure::new(
            Exit::Authentication,
            "Pipedream denied access. The CLI read 2 pages before the failure. Check that the \
             OAuth client can use this project and environment."
        )
    );
    let unavailable = pipedream_connect::error::Error::Unavailable {
        status: None,
        retry_after: None,
    };
    assert_eq!(
        interrupted(0, PageRequest::default(), unavailable).message,
        "The CLI could not reach Pipedream at https://api.pipedream.com. Check the network and \
         the API origin, then try again."
    );
}

#[test]
fn resume_hints_follow_the_cursor_direction_and_quote_the_cursor() {
    let before = PageRequest {
        before: Some(Cursor::new("cursor_0").unwrap()),
        ..PageRequest::default()
    };
    assert_eq!(
        resume(&before, None).as_deref(),
        Some("To continue, run the command again with --before 'cursor_0'.")
    );
    assert_eq!(
        resume(&after("it's$x"), NonZeroU64::new(1)).as_deref(),
        Some("Wait 1 second, then run the command again with --after 'it'\\''s$x'.")
    );
    assert_eq!(resume(&PageRequest::default(), None), None);
    assert_eq!(count(1), "1 page");
    assert_eq!(count(4), "4 pages");
}

#[test]
fn internal_failures_name_the_issue_tracker() {
    let error = Error::internal(io::Error::other("encoder detail"));
    let failure = describe(&error, DEFAULT_ORIGIN);
    assert_eq!(failure.exit, Exit::Internal);
    assert_eq!(
        failure.message,
        "The CLI could not complete the command. Report this problem with the command that you \
         ran at https://github.com/futex-ai/pipedream-connectors/issues."
    );
}

#[test]
fn a_result_that_does_not_encode_says_whether_the_change_happened() {
    let unencodable = |target| Error::Unencodable {
        target,
        cause: serde_json::from_str::<u8>("x").unwrap_err(),
    };
    assert_eq!(
        describe(&unencodable(Target::TokenCreate), DEFAULT_ORIGIN),
        Failure::new(
            Exit::Internal,
            "The CLI could not print the result. Pipedream created the token. Run the command \
             again for a new one."
        )
    );
    assert_eq!(
        describe(&unencodable(Target::Apps), DEFAULT_ORIGIN),
        Failure::new(
            Exit::Internal,
            "The CLI could not print the result. Report this problem with the command that you \
             ran at https://github.com/futex-ai/pipedream-connectors/issues."
        )
    );
}
