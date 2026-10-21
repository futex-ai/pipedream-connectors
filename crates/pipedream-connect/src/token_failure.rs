//! Replay one refresh failure to queued callers without cloning internal sources.

use std::sync::Arc;

use crate::error::Error;

#[track_caller]
pub(crate) fn repeat_failure(error: &Arc<Error>) -> Error {
    match error.as_ref() {
        Error::Unauthorized { message } => Error::Unauthorized {
            message: message.clone(),
        },
        Error::Forbidden { message } => Error::Forbidden {
            message: message.clone(),
        },
        Error::NotFound { message } => Error::NotFound {
            message: message.clone(),
        },
        Error::RateLimited {
            retry_after,
            limit,
            remaining,
            reset,
        } => Error::RateLimited {
            retry_after: *retry_after,
            limit: *limit,
            remaining: *remaining,
            reset: *reset,
        },
        Error::Rejected {
            status,
            code,
            message,
        } => Error::Rejected {
            status: *status,
            code: code.clone(),
            message: message.clone(),
        },
        Error::Unavailable { status } => Error::Unavailable { status: *status },
        Error::ResponseTooLarge { limit } => Error::ResponseTooLarge { limit: *limit },
        Error::ExecutionFailed(execution) => Error::ExecutionFailed(execution.clone()),
        Error::Accepted => Error::Accepted,
        Error::Internal(_) | Error::InvalidRequest(_) => Error::internal(error.clone()),
    }
}
