//! The exit status and the message of each command failure.

use pipedream_connect_types::PageRequest;

use crate::error::{Error, Interruption};
use crate::report::exit::Exit;
use crate::report::target::{Sent, after_request};
use crate::report::{api, input};

/// The action for a failure that the user cannot fix.
pub(crate) const REPORT: &str = "Report this problem with the command that you ran at \
     https://github.com/futex-ai/pipedream-connectors/issues.";

/// One failure as the user reads it.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Failure {
    /// The exit status.
    pub(crate) exit: Exit,
    /// One message that names what failed and a recovery action.
    pub(crate) message: String,
}

impl Failure {
    /// Pairs an exit status with its message.
    pub(crate) fn new(exit: Exit, message: impl Into<String>) -> Self {
        Self {
            exit,
            message: message.into(),
        }
    }
}

/// Describes `error` without provider text, response bodies, or credentials.
pub(crate) fn describe(error: &Error, origin: &str) -> Failure {
    match error {
        Error::MissingSettings { settings } => usage(input::missing(settings)),
        Error::InvalidSetting { setting, supply } => {
            usage(input::invalid_setting(*setting, *supply))
        }
        Error::UnreadableVariable { setting } => usage(input::unreadable_variable(*setting)),
        Error::InvalidCredentials {
            client_id,
            client_secret,
        } => usage(input::invalid_credentials(*client_id, *client_secret)),
        Error::InvalidArgument { argument, cause } => {
            usage(input::invalid_argument(*argument, cause))
        }
        Error::InvalidJson {
            argument,
            line,
            column,
            problem,
        } => usage(input::invalid_json(*argument, *line, *column, *problem)),
        Error::InvalidHeader { position, problem } => {
            usage(input::invalid_header(*position, problem))
        }
        Error::InvalidTime { argument } => usage(format!(
            "{argument} must be Unix seconds or an RFC 3339 time, such as 2026-10-01T00:00:00Z."
        )),
        Error::InvalidCount { argument } => {
            usage(format!("{argument} must be a whole number of 1 or more."))
        }
        Error::UnreadableInput { argument, cause } => {
            usage(input::unreadable(*argument, cause.kind()))
        }
        Error::Client(cause) => api::client(cause),
        Error::Api { target, cause } => api::describe(*target, cause, origin),
        Error::Interrupted(interruption) => interrupted(interruption, origin),
        Error::Unencodable { target, .. } => Failure::new(
            Exit::Internal,
            format!(
                "The CLI could not print the result. {}",
                after_request(*target, Sent::Completed).unwrap_or(REPORT)
            ),
        ),
        Error::Internal(_) => Failure::new(
            Exit::Internal,
            format!("The CLI could not complete the command. {REPORT}"),
        ),
    }
}

/// Describes a failed listing with one action.
///
/// A retryable failure after at least one page gives the command that continues
/// from the failed page. Other failures keep their own action.
fn interrupted(interruption: &Interruption, origin: &str) -> Failure {
    let text = api::explain(interruption.target, &interruption.cause, origin);
    if interruption.pages == 0 {
        return Failure::new(text.exit, format!("{} {}", text.summary, text.action));
    }
    let action = match (text.retryable, resume(&interruption.request, text.wait)) {
        (true, Some(resume)) => resume,
        _ => text.action,
    };
    Failure::new(
        text.exit,
        format!(
            "{} The CLI read {} before the failure. {action}",
            text.summary,
            count(interruption.pages)
        ),
    )
}

/// Returns the cursor flag of `request` and its cursor, quoted for a POSIX shell.
pub(crate) fn cursor_flag(request: &PageRequest) -> Option<(&'static str, String)> {
    let (flag, cursor) = match (&request.after, &request.before) {
        (Some(cursor), _) => ("--after", cursor),
        (None, Some(cursor)) => ("--before", cursor),
        (None, None) => return None,
    };
    Some((
        flag,
        format!("'{}'", cursor.as_str().replace('\'', "'\\''")),
    ))
}

/// Returns the command that continues a listing from `request`, when it has a cursor.
///
/// A rate-limit `wait` comes first.
pub(crate) fn resume(request: &PageRequest, wait: Option<u64>) -> Option<String> {
    let (flag, cursor) = cursor_flag(request)?;
    Some(match wait {
        Some(wait) => format!(
            "Wait {}, then run the command again with {flag} {cursor}.",
            seconds(wait)
        ),
        None => format!("To continue, run the command again with {flag} {cursor}."),
    })
}

/// Returns a wait in words, such as `1 second` or `30 seconds`.
pub(crate) fn seconds(count: u64) -> String {
    if count == 1 {
        "1 second".to_owned()
    } else {
        format!("{count} seconds")
    }
}

/// Returns a page count in words, such as `1 page` or `3 pages`.
pub(crate) fn count(pages: usize) -> String {
    if pages == 1 {
        "1 page".to_owned()
    } else {
        format!("{pages} pages")
    }
}

fn usage(message: String) -> Failure {
    Failure::new(Exit::Usage, message)
}

#[cfg(test)]
#[path = "_tests_/failure_tests.rs"]
mod failure_tests;
