//! Replay one refresh failure to queued callers without cloning internal sources.

use std::sync::Arc;

use crate::error::{Error, InvalidRequest};

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
        Error::InvalidRequest(reason) => Error::InvalidRequest(repeat_invalid(reason, error)),
        Error::Internal(_) => Error::internal(error.clone()),
    }
}

fn repeat_invalid(reason: &InvalidRequest, original: &Arc<Error>) -> InvalidRequest {
    match reason {
        InvalidRequest::Origin => InvalidRequest::Origin,
        InvalidRequest::Credentials => InvalidRequest::Credentials,
        InvalidRequest::Bearer => InvalidRequest::Bearer,
        InvalidRequest::Environment => InvalidRequest::Environment,
        InvalidRequest::Header => InvalidRequest::Header,
        InvalidRequest::MissingConfiguration { field } => {
            InvalidRequest::MissingConfiguration { field: *field }
        }
        InvalidRequest::Model(source) => InvalidRequest::Model(repeat_model(source, original)),
    }
}

#[track_caller]
fn repeat_model(
    source: &pipedream_connect_types::Error,
    original: &Arc<Error>,
) -> pipedream_connect_types::Error {
    match source {
        pipedream_connect_types::Error::InvalidIdentifier { kind } => {
            pipedream_connect_types::Error::InvalidIdentifier { kind: *kind }
        }
        pipedream_connect_types::Error::OutOfRange {
            field,
            minimum,
            maximum,
        } => pipedream_connect_types::Error::OutOfRange {
            field: *field,
            minimum: *minimum,
            maximum: *maximum,
        },
        pipedream_connect_types::Error::ConflictingCursors => {
            pipedream_connect_types::Error::ConflictingCursors
        }
        pipedream_connect_types::Error::StuckCursor => pipedream_connect_types::Error::StuckCursor,
        pipedream_connect_types::Error::InvalidTimestamp => {
            pipedream_connect_types::Error::InvalidTimestamp
        }
        pipedream_connect_types::Error::InvalidHttpsUrl { field } => {
            pipedream_connect_types::Error::InvalidHttpsUrl { field: *field }
        }
        pipedream_connect_types::Error::Internal(_) => {
            pipedream_connect_types::Error::internal(original.clone())
        }
    }
}
