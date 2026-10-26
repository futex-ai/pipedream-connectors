//! The wait that an unavailable or rate-limited answer asks for.
//!
//! A wait is a whole number of seconds that is never zero, so no text can ask
//! for a zero wait.

use std::num::NonZeroU64;
use std::time::Duration;

use crate::report::api::ApiText;
use crate::report::exit::Exit;
use crate::report::failure::seconds;

/// Waits for the longer of the `Retry-After` and `X-PD-Rate-Limit-Reset-In` delays.
pub(super) fn rate_limited(
    retry_after: Option<Duration>,
    reset_after: Option<Duration>,
) -> ApiText {
    let wait = [retry_after, reset_after]
        .into_iter()
        .flatten()
        .filter_map(whole_seconds)
        .max();
    let action = match wait {
        Some(wait) => format!("Wait {}, then run the command again.", seconds(wait)),
        None => "Wait a moment, then run the command again.".to_owned(),
    };
    ApiText {
        exit: Exit::RateLimited,
        summary: "Pipedream rate limited the request.".to_owned(),
        action,
        retryable: true,
        wait,
    }
}

/// Names the `Retry-After` wait of a 408 or 5xx answer, when it has one.
pub(super) fn unavailable(
    status: Option<u16>,
    retry_after: Option<Duration>,
    origin: &str,
) -> ApiText {
    let Some(status) = status else {
        return ApiText {
            exit: Exit::Unavailable,
            summary: format!("The CLI could not reach Pipedream at {origin}."),
            action: "Check the network and the API origin, then try again.".to_owned(),
            retryable: true,
            wait: None,
        };
    };
    let wait = retry_after.and_then(whole_seconds);
    let action = match wait {
        Some(wait) => format!("Wait {}, then try again.", seconds(wait)),
        None => "Try again later.".to_owned(),
    };
    ApiText {
        exit: Exit::Unavailable,
        summary: format!("Pipedream at {origin} is unavailable with status {status}."),
        action,
        retryable: true,
        wait,
    }
}

/// Rounds a delay up to whole seconds. A zero delay asks for no wait.
fn whole_seconds(delay: Duration) -> Option<NonZeroU64> {
    NonZeroU64::new(
        delay
            .as_secs()
            .saturating_add(u64::from(delay.subsec_nanos() > 0)),
    )
}
