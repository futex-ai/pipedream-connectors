//! Tests for the exit status and the text of configuration and input failures.

mod argument_tests;
mod hint_tests;
mod setting_tests;

use crate::argument::Argument;
use crate::error::Error;
use crate::report::exit::Exit;
use crate::report::failure::{Failure, describe};
use crate::settings::DEFAULT_ORIGIN;

fn usage(message: &str) -> Failure {
    Failure::new(Exit::Usage, message)
}

fn argument(argument: Argument, cause: pipedream_connect_types::Error) -> Failure {
    describe(&Error::InvalidArgument { argument, cause }, DEFAULT_ORIGIN)
}
