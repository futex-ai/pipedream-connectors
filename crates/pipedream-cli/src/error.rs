//! Failures that stop one command, with the typed context that the report needs.

use std::io;

use internal_error::InternalError;
use pipedream_connect_types::PageRequest;
use thiserror::Error;

use crate::argument::Argument;
use crate::settings::{Setting, Supply};
use crate::target::Target;

/// Why a JSON input did not decode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum JsonProblem {
    /// The input is not JSON.
    Syntax,
    /// The JSON does not have the expected shape.
    Shape,
}

/// A failure that stops one command.
#[derive(Debug, Error, internal_error::ErrorContract)]
pub(crate) enum Error {
    /// Required settings have neither a flag nor a variable.
    #[error("[pipedream-cli/settings] {count} required settings are missing", count = .settings.len())]
    MissingSettings {
        /// The missing settings, in resolution order.
        settings: Vec<Setting>,
    },
    /// A setting value does not satisfy its grammar.
    #[error("[pipedream-cli/settings] invalid {setting} from its {supply}")]
    InvalidSetting {
        /// The invalid setting.
        setting: Setting,
        /// How the caller supplied it.
        supply: Supply,
    },
    /// A variable holds bytes that are not UTF-8 text.
    #[error("[pipedream-cli/settings] {setting} is not UTF-8")]
    UnreadableVariable {
        /// The unreadable setting.
        setting: Setting,
    },
    /// The client ID or the client secret does not satisfy the SDK grammar.
    #[error("[pipedream-cli/settings] invalid client credentials")]
    InvalidCredentials {
        /// How the caller supplied the client ID.
        client_id: Supply,
        /// How the caller supplied the client secret.
        client_secret: Supply,
    },
    /// A command argument does not satisfy its model grammar.
    #[error("[pipedream-cli/input] invalid {argument}")]
    InvalidArgument {
        /// The invalid argument.
        argument: Argument,
        /// The model validation failure.
        #[source]
        cause: pipedream_connect_types::Error,
    },
    /// A JSON input did not decode.
    #[error("[pipedream-cli/input] invalid {argument} JSON at {line}:{column}")]
    InvalidJson {
        /// The JSON argument.
        argument: Argument,
        /// The line of the failure.
        line: usize,
        /// The column of the failure.
        column: usize,
        /// Why the input did not decode.
        problem: JsonProblem,
    },
    /// An input file or standard input could not be read.
    #[error("[pipedream-cli/input] cannot read {argument}")]
    UnreadableInput {
        /// The input argument.
        argument: Argument,
        /// The read failure.
        #[source]
        cause: io::Error,
    },
    /// The SDK rejected the client configuration or could not build the client.
    #[error("[pipedream-cli/client] client construction failed")]
    Client(#[source] pipedream_connect::error::Error),
    /// One API operation failed.
    #[error("[pipedream-cli/api] {target} request failed")]
    Api {
        /// The resource of the operation.
        target: Target,
        /// The SDK failure.
        #[source]
        cause: pipedream_connect::error::Error,
    },
    /// A multi-page listing failed after zero or more completed pages.
    #[error("[pipedream-cli/api] listing interrupted")]
    Interrupted(#[source] Box<Interruption>),
    /// An internal failure, such as an output encoding failure.
    #[error("[pipedream-cli/internal] internal error")]
    Internal(#[from] InternalError),
}

/// A multi-page listing that failed, and the request that can resume it.
#[derive(Debug, Error)]
#[error("[pipedream-cli/api] {target} failed after {pages} pages")]
pub(crate) struct Interruption {
    /// The listed resource.
    pub(crate) target: Target,
    /// The number of completed pages.
    pub(crate) pages: usize,
    /// The page request that failed.
    pub(crate) request: PageRequest,
    /// The SDK failure.
    #[source]
    pub(crate) cause: pipedream_connect::error::Error,
}

/// Result type for CLI commands.
pub(crate) type Result<T> = std::result::Result<T, Error>;

/// Converts the model validation of `argument` into a typed input error.
pub(crate) fn checked<T>(
    result: pipedream_connect_types::Result<T>,
    argument: Argument,
) -> Result<T> {
    match result {
        Ok(value) => Ok(value),
        Err(cause) => Err(Error::InvalidArgument { argument, cause }),
    }
}

/// Converts the SDK result of an operation on `target` into a typed API error.
pub(crate) fn called<T>(result: pipedream_connect::error::Result<T>, target: Target) -> Result<T> {
    match result {
        Ok(value) => Ok(value),
        Err(cause) => Err(Error::Api { target, cause }),
    }
}
