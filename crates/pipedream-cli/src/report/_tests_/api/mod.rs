//! Tests for the exit status and the text of each SDK failure.

mod status_tests;
mod uncertain_tests;
mod wait_tests;

use pipedream_connect::error::Error;
use pipedream_connect_types::ExecutionError;

use crate::report::api::describe;
use crate::report::failure::Failure;
use crate::settings::DEFAULT_ORIGIN;
use crate::target::Target;

pub(super) const PROVIDER_TEXT: &str = "provider text with token xoxb-hidden";

pub(super) const REPORT: &str = "Report this problem with the command that you ran at \
                      https://github.com/futex-ai/pipedream-connectors/issues.";

pub(super) const MAY_HAVE_RUN: &str =
    "The action may have run. Check its effects before you run it again.";

pub(super) fn text(target: Target, cause: &Error) -> Failure {
    describe(target, cause, DEFAULT_ORIGIN)
}

pub(super) fn execution(error: serde_json::Value) -> Error {
    let error: ExecutionError = serde_json::from_value(error).unwrap();
    Error::ExecutionFailed {
        status: 400,
        error: Box::new(error),
    }
}
