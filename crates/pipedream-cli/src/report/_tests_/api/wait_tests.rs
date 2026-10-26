//! Tests for the wait that an unavailable or rate-limited answer asks for.

use std::time::Duration;

use pipedream_connect::error::Error;

use crate::report::api::describe;
use crate::report::exit::Exit;
use crate::report::failure::Failure;
use crate::target::Target;

use super::{MAY_HAVE_RUN, text};

fn unavailable(retry_after: Option<Duration>) -> Error {
    Error::Unavailable {
        status: Some(503),
        retry_after,
    }
}

#[test]
fn an_unavailable_answer_with_a_server_delay_names_the_wait() {
    let cases = [
        (
            Some(Duration::from_secs(30)),
            "Wait 30 seconds, then try again.",
        ),
        (Some(Duration::ZERO), "Try again later."),
        (None, "Try again later."),
    ];
    for (retry_after, action) in cases {
        assert_eq!(
            describe(
                Target::Apps,
                &unavailable(retry_after),
                "https://api.example.com"
            ),
            Failure::new(
                Exit::Unavailable,
                format!(
                    "Pipedream at https://api.example.com is unavailable with status 503. {action}"
                )
            )
        );
    }
}

#[test]
fn a_change_keeps_its_warning_instead_of_the_wait() {
    assert_eq!(
        text(
            Target::ActionRun,
            &unavailable(Some(Duration::from_secs(30)))
        ),
        Failure::new(
            Exit::Unavailable,
            format!(
                "Pipedream at https://api.pipedream.com is unavailable with status 503. \
                 {MAY_HAVE_RUN}"
            )
        )
    );
}

#[test]
fn a_rate_limit_waits_for_the_longer_delay() {
    let limited = |retry_after, reset_after| Error::RateLimited {
        retry_after,
        reset_after,
        limit: None,
        remaining: None,
        reset: None,
    };
    let seconds = |count| Some(Duration::from_secs(count));
    let cases = [
        (None, seconds(45), "Wait 45 seconds"),
        (seconds(10), seconds(45), "Wait 45 seconds"),
        (seconds(60), seconds(45), "Wait 60 seconds"),
        (Some(Duration::ZERO), Some(Duration::ZERO), "Wait a moment"),
    ];
    for (retry_after, reset_after, wait) in cases {
        assert_eq!(
            text(Target::Users, &limited(retry_after, reset_after)),
            Failure::new(
                Exit::RateLimited,
                format!("Pipedream rate limited the request. {wait}, then run the command again.")
            ),
            "{retry_after:?} {reset_after:?}"
        );
    }
}
