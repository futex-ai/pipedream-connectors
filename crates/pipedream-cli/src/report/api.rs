//! User-facing text for SDK failures, chosen by the resource of the command.

use std::time::Duration;

use pipedream_connect::error::{Error, InvalidRequest};
use pipedream_connect_types::{ErrorOrigin, ExecutionError};

use crate::report::exit::Exit;
use crate::report::failure::Failure;
use crate::report::target::text;
use crate::target::Target;

/// Describes an SDK failure without provider messages, provider codes, or bodies.
pub(super) fn describe(target: Target, cause: &Error) -> Failure {
    match cause {
        Error::InvalidRequest(reason) => Failure::new(Exit::Usage, invalid_request(reason)),
        Error::Unauthorized { .. } => Failure::new(
            Exit::Authentication,
            "Pipedream rejected the client credentials. Check the client ID and the client secret.",
        ),
        Error::Forbidden { .. } => Failure::new(
            Exit::Authentication,
            "Pipedream denied access. Check that the OAuth client can use this project and environment.",
        ),
        Error::NotFound { .. } => Failure::new(Exit::NotFound, text(target).not_found),
        Error::RateLimited { retry_after, .. } => {
            Failure::new(Exit::RateLimited, rate_limited(*retry_after))
        }
        Error::Rejected { status, .. } => Failure::new(
            Exit::Rejected,
            format!(
                "Pipedream rejected the request with status {status}. Check the command arguments."
            ),
        ),
        Error::ExecutionFailed { error, .. } => {
            Failure::new(Exit::Rejected, execution_failed(target, error))
        }
        Error::Accepted => Failure::new(
            Exit::Rejected,
            "Pipedream accepted the request without a result. Run the command again later.",
        ),
        Error::Unavailable { status } => {
            Failure::new(Exit::Unavailable, unavailable(target, *status))
        }
        Error::ResponseTooLarge { limit } => {
            Failure::new(Exit::Rejected, too_large(target, *limit))
        }
        Error::Internal(_) => Failure::new(
            Exit::Internal,
            "Pipedream returned a response that the CLI cannot read. \
             Report this problem with the command that you ran.",
        ),
    }
}

/// Describes a failure to configure or build the SDK client.
pub(super) fn client(cause: &Error) -> Failure {
    match cause {
        Error::InvalidRequest(reason) => Failure::new(Exit::Usage, invalid_request(reason)),
        _ => Failure::new(
            Exit::Internal,
            "The CLI could not start its HTTP client. Run the command again.",
        ),
    }
}

fn invalid_request(reason: &InvalidRequest) -> &'static str {
    match reason {
        InvalidRequest::Header => {
            "A request header is not valid. Check the header names and values."
        }
        InvalidRequest::Model(_) => "A request value is not valid. Check the command arguments.",
        InvalidRequest::Origin
        | InvalidRequest::Credentials
        | InvalidRequest::Bearer
        | InvalidRequest::Environment
        | InvalidRequest::MissingConfiguration { .. } => {
            "The connection settings are not valid. Check the configuration flags and variables."
        }
    }
}

fn rate_limited(retry_after: Option<Duration>) -> String {
    match retry_after {
        Some(delay) => {
            let seconds = delay.as_secs() + u64::from(delay.subsec_nanos() > 0);
            let unit = if seconds == 1 { "second" } else { "seconds" };
            format!(
                "Pipedream rate limited the request. Wait {seconds} {unit}, then run the command again."
            )
        }
        None => "Pipedream rate limited the request. Wait a moment, then run the command again."
            .to_owned(),
    }
}

fn execution_failed(target: Target, error: &ExecutionError) -> String {
    let subject = text(target).subject;
    let place = match error.attribution.as_ref() {
        Some(attribution) => match &attribution.origin {
            ErrorOrigin::ComponentCode => " in its component code".to_owned(),
            ErrorOrigin::UpstreamApi => {
                match attribution.last_call.as_ref().and_then(|call| call.status) {
                    Some(status) => format!(" at the app API with status {status}"),
                    None => " at the app API".to_owned(),
                }
            }
            ErrorOrigin::NetworkIo => " on a network call".to_owned(),
            ErrorOrigin::ResponseParsing => " while it read an app response".to_owned(),
            ErrorOrigin::Other(_) => String::new(),
        },
        None => String::new(),
    };
    format!("{subject} failed{place}. Check the configured props, then run the command again.")
}

fn unavailable(target: Target, status: Option<u16>) -> String {
    let summary = match status {
        Some(status) => format!("Pipedream is unavailable with status {status}."),
        None => "Pipedream did not respond.".to_owned(),
    };
    match text(target).unanswered {
        Some(warning) => format!("{summary} {warning}"),
        None => format!("{summary} Try again later."),
    }
}

fn too_large(target: Target, limit: usize) -> String {
    let size = if limit.is_multiple_of(1024 * 1024) {
        format!("{} MiB", limit / (1024 * 1024))
    } else {
        format!("{limit} bytes")
    };
    if text(target).paged {
        format!("The Pipedream response is larger than {size}. Request fewer records with --limit.")
    } else {
        format!("The Pipedream response is larger than the {size} limit of the CLI.")
    }
}

#[cfg(test)]
#[path = "_tests_/api_tests.rs"]
mod api_tests;
