//! Typed failures at the client boundary, with safe diagnostic formatting.

use std::fmt;
use std::time::Duration;

use internal_error::InternalError;
use pipedream_connect_types::ExecutionError;
use thiserror::Error;

/// A required client-builder setting.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigurationField {
    /// Project identifier.
    ProjectId,
    /// Explicit environment.
    Environment,
    /// Client credentials or static bearer.
    Authentication,
}

impl fmt::Display for ConfigurationField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ProjectId => "project_id",
            Self::Environment => "environment",
            Self::Authentication => "authentication",
        })
    }
}

/// A local input error. No HTTP request has been sent.
#[derive(Error)]
pub enum InvalidRequest {
    /// The configured origin is not a bare HTTPS origin.
    #[error("[pipedream-connect/config] expected a bare HTTPS origin")]
    Origin,
    /// Credentials are empty, padded, or contain control characters.
    #[error("[pipedream-connect/config] invalid client credentials")]
    Credentials,
    /// A bearer cannot be used in an authorization header.
    #[error("[pipedream-connect/config] invalid bearer")]
    Bearer,
    /// Only development and production are request environments.
    #[error("[pipedream-connect/config] invalid request environment")]
    Environment,
    /// A header value cannot be sent over HTTP.
    #[error("[pipedream-connect/request] invalid header value")]
    Header,
    /// A workflow invocation uses an unsupported HTTP method.
    #[error("[pipedream-connect/request] unsupported workflow method")]
    WorkflowMethod,
    /// The builder is missing a required setting.
    #[error("[pipedream-connect/config] missing {field}")]
    MissingConfiguration {
        /// Name of the required setting.
        field: ConfigurationField,
    },
    /// A typed request failed its model validation.
    #[error("[pipedream-connect/request] invalid request model: {0}")]
    Model(pipedream_connect_types::Error),
}

impl fmt::Debug for InvalidRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

/// Public client failure categories.
#[derive(Error, internal_error::ErrorContract)]
pub enum Error {
    /// Local input validation failed before authentication.
    #[error("[pipedream-connect/request] invalid request")]
    InvalidRequest(#[source] InvalidRequest),
    /// The API rejected the bearer.
    #[error("[pipedream-connect/http] unauthorized")]
    Unauthorized {
        /// Provider message, when supplied. Never printed by this error.
        message: Option<String>,
    },
    /// The API denied access.
    #[error("[pipedream-connect/http] forbidden")]
    Forbidden {
        /// Provider message, when supplied. Never printed by this error.
        message: Option<String>,
    },
    /// The requested resource does not exist.
    #[error("[pipedream-connect/http] not found")]
    NotFound {
        /// Provider message, when supplied. Never printed by this error.
        message: Option<String>,
    },
    /// The API rejected the request because of a rate limit.
    #[error("[pipedream-connect/http] rate limited")]
    RateLimited {
        /// Parsed Retry-After delay.
        retry_after: Option<Duration>,
        /// Request limit.
        limit: Option<u64>,
        /// Remaining requests.
        remaining: Option<u64>,
        /// Reset time as Unix seconds.
        reset: Option<u64>,
    },
    /// An unhandled HTTP status rejected the request.
    #[error("[pipedream-connect/http] request rejected with status {status}")]
    Rejected {
        /// HTTP status code.
        status: u16,
        /// Provider error code, when supplied.
        code: Option<String>,
        /// Provider message, when supplied. Never printed by this error.
        message: Option<String>,
    },
    /// An execution failed with provider attribution.
    #[error("[pipedream-connect/execution] execution failed with status {status}")]
    ExecutionFailed {
        /// HTTP status returned by Connect.
        status: u16,
        /// Structured provider execution details.
        error: Box<ExecutionError>,
    },
    /// A nonblocking execution was accepted for later completion.
    #[error("[pipedream-connect/execution] accepted")]
    Accepted,
    /// A timeout, transport error, 408, or 5xx occurred. The request may have run.
    #[error("[pipedream-connect/http] unavailable, status {status:?}")]
    Unavailable {
        /// HTTP status, if a response was received.
        status: Option<u16>,
    },
    /// A response exceeded its configured byte limit.
    #[error("[pipedream-connect/http] response exceeded {limit} bytes")]
    ResponseTooLarge {
        /// Configured response byte limit.
        limit: usize,
    },
    /// An encoding, decoding, or construction failure.
    #[error("[pipedream-connect/internal] internal client error")]
    Internal(#[from] InternalError),
}

/// Result type at every client runtime boundary.
pub type Result<T> = std::result::Result<T, Error>;

/// Converts model validation without retaining request values.
pub(crate) fn validate<T>(result: pipedream_connect_types::Result<T>) -> Result<T> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => Err(Error::InvalidRequest(InvalidRequest::Model(error))),
    }
}

#[derive(Debug, Error)]
pub(crate) enum WireFailure {
    #[error("[pipedream-connect/json] invalid response JSON at {line}:{column}")]
    Decode { line: usize, column: usize },
    #[error("[pipedream-connect/json] request encoding failed")]
    Encode,
    #[error("[pipedream-connect/http] client construction failed")]
    #[cfg(feature = "reqwest")]
    ClientConstruction,
    #[error("[pipedream-connect/http] request construction failed")]
    #[cfg(feature = "reqwest")]
    RequestConstruction,
    #[error("[pipedream-connect/token] invalid access token or lifetime")]
    Token,
    #[error("[pipedream-connect/request] URL construction failed")]
    Url,
    #[error("[pipedream-connect/request] query model has a nested value")]
    QueryShape,
    #[error("[pipedream-connect/resources] response identity does not match the request")]
    IdentityMismatch,
    #[error("[pipedream-connect/resources] response exceeded the requested event count")]
    Pagination,
    #[error("[pipedream-connect/resources] redirect response has no location")]
    MissingLocation,
}
