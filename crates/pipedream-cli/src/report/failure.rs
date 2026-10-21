//! The exit status and the message of each command failure.

use pipedream_connect_types::PageRequest;

use crate::error::Error;
use crate::report::exit::Exit;
use crate::report::{api, input};

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
pub(crate) fn describe(error: &Error) -> Failure {
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
        Error::UnreadableInput { argument, cause } => {
            usage(input::unreadable(*argument, cause.kind()))
        }
        Error::Client(cause) => api::client(cause),
        Error::Api { target, cause } => api::describe(*target, cause),
        Error::Interrupted(interruption) => {
            let failure = api::describe(interruption.target, &interruption.cause);
            let message = match (interruption.pages, resume(&interruption.request)) {
                (0, _) | (_, None) => failure.message,
                (pages, Some(resume)) => format!(
                    "{} The CLI read {} before the failure. {resume}",
                    failure.message,
                    count(pages)
                ),
            };
            Failure::new(failure.exit, message)
        }
        Error::Internal(_) => Failure::new(
            Exit::Internal,
            "The CLI could not complete the command. Report this problem with the command that you ran.",
        ),
    }
}

/// Returns the flag that continues a listing from `request`, when it has a cursor.
pub(crate) fn resume(request: &PageRequest) -> Option<String> {
    match (&request.after, &request.before) {
        (Some(cursor), _) => Some(format!(
            "To continue, run the command again with --after {cursor}."
        )),
        (None, Some(cursor)) => Some(format!(
            "To continue, run the command again with --before {cursor}."
        )),
        (None, None) => None,
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
