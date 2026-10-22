//! The documented process exit statuses.

use std::process::ExitCode;

/// The process exit status of one CLI run.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Exit {
    /// The command printed its result.
    Success,
    /// The CLI could not complete the command.
    Internal,
    /// A flag, a variable, or an input is not valid. No request was sent.
    Usage,
    /// Pipedream rejected the credentials or denied access.
    Authentication,
    /// Pipedream did not find the requested resource.
    NotFound,
    /// Pipedream rate limited the request.
    RateLimited,
    /// Pipedream was unavailable or did not respond. The request may have run.
    Unavailable,
    /// Pipedream rejected the request, or the action failed.
    Rejected,
}

impl Exit {
    /// Returns the documented numeric exit code.
    pub(crate) fn code(self) -> u8 {
        match self {
            Self::Success => 0,
            Self::Internal => 1,
            Self::Usage => 2,
            Self::Authentication => 3,
            Self::NotFound => 4,
            Self::RateLimited => 5,
            Self::Unavailable => 6,
            Self::Rejected => 7,
        }
    }
}

impl From<Exit> for ExitCode {
    fn from(exit: Exit) -> Self {
        Self::from(exit.code())
    }
}
