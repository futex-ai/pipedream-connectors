//! User-facing text for SDK failures, chosen by the resource of the command.

use std::time::Duration;

use pipedream_connect::error::{Error, InvalidRequest};
use pipedream_connect_types::{ErrorOrigin, ExecutionError};

use crate::report::exit::Exit;
use crate::report::failure::{Failure, REPORT, seconds};
use crate::report::target::text;
use crate::target::Target;

/// An SDK failure, split into what failed and the one recovery action.
#[derive(Debug, Eq, PartialEq)]
pub(super) struct ApiText {
    /// The exit status.
    pub(super) exit: Exit,
    /// What failed.
    pub(super) summary: String,
    /// The one recovery action.
    pub(super) action: String,
    /// Whether the same request can succeed later without a change.
    pub(super) retryable: bool,
    /// The wait that a rate limit asks for, in seconds.
    pub(super) wait: Option<u64>,
}

/// Describes an SDK failure without provider messages, provider codes, or bodies.
pub(super) fn describe(target: Target, cause: &Error, origin: &str) -> Failure {
    let text = explain(target, cause, origin);
    Failure::new(text.exit, format!("{} {}", text.summary, text.action))
}

/// Splits an SDK failure into its summary and its action.
///
/// When a command changes data and the answer is missing or unreadable, the
/// action becomes the warning that the change may have happened.
pub(super) fn explain(target: Target, cause: &Error, origin: &str) -> ApiText {
    let mut explained = base(target, cause, origin);
    let unknown = matches!(
        cause,
        Error::Unavailable { .. } | Error::ResponseTooLarge { .. } | Error::Internal(_)
    );
    if let (true, Some(warning)) = (unknown, text(target).uncertain) {
        explained.action = warning.to_owned();
        explained.retryable = false;
    }
    explained
}

/// Describes a failure to configure or build the SDK client.
pub(super) fn client(cause: &Error) -> Failure {
    match cause {
        Error::InvalidRequest(reason) => {
            let (summary, action) = invalid_request(reason);
            Failure::new(Exit::Usage, format!("{summary} {action}"))
        }
        _ => Failure::new(
            Exit::Internal,
            "The CLI could not start its HTTP client. Run the command again.",
        ),
    }
}

fn base(target: Target, cause: &Error, origin: &str) -> ApiText {
    let target_text = text(target);
    match cause {
        Error::InvalidRequest(reason) => {
            let (summary, action) = invalid_request(reason);
            fixed(Exit::Usage, summary, action)
        }
        Error::Unauthorized { .. } => fixed(
            Exit::Authentication,
            "Pipedream rejected the client credentials.",
            "Check the client ID and the client secret.",
        ),
        Error::Forbidden { .. } => fixed(
            Exit::Authentication,
            "Pipedream denied access.",
            "Check that the OAuth client can use this project and environment.",
        ),
        Error::NotFound { .. } => fixed(
            Exit::NotFound,
            target_text.missing,
            target_text.missing_action,
        ),
        Error::RateLimited { retry_after, .. } => rate_limited(*retry_after),
        Error::Rejected { status, .. } => fixed(
            Exit::Rejected,
            &format!("Pipedream rejected the request with status {status}."),
            "Check the command arguments.",
        ),
        Error::ExecutionFailed { error, .. } => fixed(
            Exit::Rejected,
            &format!("{} failed{}.", target_text.subject, place(error)),
            target_text.execution_action,
        ),
        Error::Accepted => fixed(
            Exit::Rejected,
            "Pipedream accepted the request without a result.",
            "Run the command again later.",
        ),
        Error::Unavailable { status } => unavailable(*status, origin),
        Error::ResponseTooLarge { limit } => fixed(
            Exit::Internal,
            &format!(
                "The Pipedream response is larger than {}, so the CLI could not read it.",
                size(*limit)
            ),
            if target_text.paged {
                "Request fewer records with --limit."
            } else {
                REPORT
            },
        ),
        Error::Internal(_) => fixed(
            Exit::Internal,
            "Pipedream returned a response that the CLI cannot read.",
            REPORT,
        ),
    }
}

fn fixed(exit: Exit, summary: &str, action: &str) -> ApiText {
    ApiText {
        exit,
        summary: summary.to_owned(),
        action: action.to_owned(),
        retryable: false,
        wait: None,
    }
}

fn invalid_request(reason: &InvalidRequest) -> (&'static str, &'static str) {
    match reason {
        InvalidRequest::Header => (
            "A request header is not valid.",
            "Check the header names and values.",
        ),
        InvalidRequest::Model(_) => (
            "A request value is not valid.",
            "Check the command arguments.",
        ),
        InvalidRequest::ProxyTarget => (
            "<URL> must be an HTTPS URL or a path that starts with /.",
            "Fix the proxy target.",
        ),
        InvalidRequest::ProxyHeader => (
            "Pipedream does not forward one of the --header values.",
            "Check the header names.",
        ),
        InvalidRequest::WorkflowMethod => (
            "Pipedream workflows accept GET, POST, PUT, PATCH, and DELETE.",
            "Use one of them.",
        ),
        InvalidRequest::Origin
        | InvalidRequest::Credentials
        | InvalidRequest::Bearer
        | InvalidRequest::Environment
        | InvalidRequest::MissingConfiguration { .. } => (
            "The connection settings are not valid.",
            "Check the configuration flags and variables.",
        ),
    }
}

fn rate_limited(retry_after: Option<Duration>) -> ApiText {
    let wait = retry_after.map(|delay| delay.as_secs() + u64::from(delay.subsec_nanos() > 0));
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

fn unavailable(status: Option<u16>, origin: &str) -> ApiText {
    let (summary, action) = match status {
        Some(status) => (
            format!("Pipedream at {origin} is unavailable with status {status}."),
            "Try again later.",
        ),
        None => (
            format!("The CLI could not reach Pipedream at {origin}."),
            "Check the network and the API origin, then try again.",
        ),
    };
    ApiText {
        exit: Exit::Unavailable,
        summary,
        action: action.to_owned(),
        retryable: true,
        wait: None,
    }
}

fn place(error: &ExecutionError) -> String {
    let Some(attribution) = error.attribution.as_ref() else {
        return String::new();
    };
    match &attribution.origin {
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
    }
}

fn size(limit: usize) -> String {
    if limit.is_multiple_of(1024 * 1024) {
        format!("{} MiB", limit / (1024 * 1024))
    } else {
        format!("{limit} bytes")
    }
}

#[cfg(test)]
#[path = "_tests_/api/mod.rs"]
mod api_tests;
