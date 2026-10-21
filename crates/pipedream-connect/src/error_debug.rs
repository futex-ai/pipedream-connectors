//! Diagnostic fields that cannot contain provider text or credentials.

use std::fmt;

use pipedream_connect_types::{ErrorOrigin, ExecutionAttribution, NetworkCall};

use crate::error::Error;

impl fmt::Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(reason) => formatter
                .debug_tuple("InvalidRequest")
                .field(reason)
                .finish(),
            Self::Unauthorized { .. } => formatter.write_str("Unauthorized"),
            Self::Forbidden { .. } => formatter.write_str("Forbidden"),
            Self::NotFound { .. } => formatter.write_str("NotFound"),
            Self::RateLimited {
                retry_after,
                limit,
                remaining,
                reset,
            } => formatter
                .debug_struct("RateLimited")
                .field("retry_after", retry_after)
                .field("limit", limit)
                .field("remaining", remaining)
                .field("reset", reset)
                .finish(),
            Self::Rejected { status, .. } => formatter
                .debug_struct("Rejected")
                .field("status", status)
                .finish_non_exhaustive(),
            Self::ExecutionFailed(error) => formatter
                .debug_struct("ExecutionFailed")
                .field(
                    "attribution",
                    &error.attribution.as_ref().map(SafeAttribution),
                )
                .finish_non_exhaustive(),
            Self::Accepted => formatter.write_str("Accepted"),
            Self::Unavailable { status } => formatter
                .debug_struct("Unavailable")
                .field("status", status)
                .finish(),
            Self::ResponseTooLarge { limit } => formatter
                .debug_struct("ResponseTooLarge")
                .field("limit", limit)
                .finish(),
            Self::Internal(source) => formatter.debug_tuple("Internal").field(source).finish(),
        }
    }
}

struct SafeAttribution<'a>(&'a ExecutionAttribution);

impl fmt::Debug for SafeAttribution<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let origin = match self.0.origin {
            ErrorOrigin::ComponentCode => "ComponentCode",
            ErrorOrigin::UpstreamApi => "UpstreamApi",
            ErrorOrigin::NetworkIo => "NetworkIo",
            ErrorOrigin::ResponseParsing => "ResponseParsing",
            ErrorOrigin::Other(_) => "Other([REDACTED])",
        };
        formatter
            .debug_struct("ExecutionAttribution")
            .field("origin", &origin)
            .field("network_calls_count", &self.0.network_calls_count)
            .field("last_call", &self.0.last_call.as_ref().map(SafeNetworkCall))
            .finish_non_exhaustive()
    }
}

struct SafeNetworkCall<'a>(&'a NetworkCall);

impl fmt::Debug for SafeNetworkCall<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NetworkCall")
            .field("status", &self.0.status)
            .field("duration_ms", &self.0.duration_ms)
            .finish_non_exhaustive()
    }
}
