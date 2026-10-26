//! Unit tests for command execution error mapping.

use std::io;
use std::path::Path;

use unimock::{MockFn, Unimock, matching};

use crate::command::{CommandOutcome, CommandRunnerMock, CommandSpec, run};
use crate::error::Error;

#[test]
fn maps_a_failed_process_to_command_failed() {
    let command = CommandSpec::inherited("cargo", &["test", "--locked"]);
    let runner = Unimock::new(
        CommandRunnerMock::execute
            .next_call(matching!(_, _))
            .returns(Ok(CommandOutcome {
                success: false,
                code: Some(23),
                stdout: Vec::new(),
            })),
    );

    let error = run(Path::new("/workspace"), &runner, &command).unwrap_err();

    assert!(matches!(
        error,
        Error::CommandFailed {
            command,
            code: Some(23)
        } if command == "cargo test --locked"
    ));
}

#[test]
fn maps_a_process_start_failure_to_command_start() {
    let command = CommandSpec::inherited("cargo", &["doc"]);
    let runner = Unimock::new(
        CommandRunnerMock::execute
            .next_call(matching!(_, _))
            .returns(Err(io::Error::new(
                io::ErrorKind::NotFound,
                "cargo missing",
            ))),
    );

    let error = run(Path::new("/workspace"), &runner, &command).unwrap_err();

    assert!(matches!(
        error,
        Error::CommandStart { command, source }
            if command == "cargo doc" && source.kind() == io::ErrorKind::NotFound
    ));
}
