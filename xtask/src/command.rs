//! Injectable external-command execution for xtask.

use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::{Error, Result};

/// A command specification passed to a command runner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandSpec {
    /// Executable name.
    pub program: String,
    /// Arguments in process order.
    pub args: Vec<String>,
    /// Optional environment variables for the process.
    pub environment: Vec<(String, String)>,
    /// Whether standard output should be captured for the caller.
    pub capture_output: bool,
}

impl CommandSpec {
    /// Creates a command that inherits the caller's output streams.
    pub(crate) fn inherited(program: &str, args: &[&str]) -> Self {
        Self {
            program: program.to_owned(),
            args: args.iter().map(|arg| (*arg).to_owned()).collect(),
            environment: Vec::new(),
            capture_output: false,
        }
    }

    /// Creates a command that captures standard output.
    pub(crate) fn captured(program: &str, args: &[&str]) -> Self {
        Self {
            capture_output: true,
            ..Self::inherited(program, args)
        }
    }

    /// Adds one environment variable to this command.
    pub(crate) fn with_environment(mut self, key: &str, value: &str) -> Self {
        self.environment.push((key.to_owned(), value.to_owned()));
        self
    }

    /// Returns a readable command label for errors.
    pub(crate) fn label(&self) -> String {
        std::iter::once(self.program.as_str())
            .chain(self.args.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Outcome returned by an executed command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandOutcome {
    /// Whether the process exited successfully.
    pub success: bool,
    /// The process exit code, when one was available.
    pub code: Option<i32>,
    /// Captured standard output.
    pub stdout: Vec<u8>,
}

/// Boundary for executing external commands.
#[cfg_attr(test, unimock::unimock(api = CommandRunnerMock))]
pub trait CommandRunner {
    /// Executes one command from the workspace root.
    fn execute(&self, root: &Path, command: &CommandSpec) -> io::Result<CommandOutcome>;
}

/// Production command runner that inherits command output.
pub(crate) struct ProcessRunner;

impl CommandRunner for ProcessRunner {
    fn execute(&self, root: &Path, command: &CommandSpec) -> io::Result<CommandOutcome> {
        let mut process = Command::new(&command.program);
        process
            .args(&command.args)
            .current_dir(root)
            .envs(command.environment.iter().map(|(key, value)| (key, value)))
            .stdin(Stdio::inherit())
            .stderr(Stdio::inherit());
        if command.capture_output {
            process.stdout(Stdio::piped());
            let output = process.spawn()?.wait_with_output()?;
            Ok(CommandOutcome {
                success: output.status.success(),
                code: output.status.code(),
                stdout: output.stdout,
            })
        } else {
            process.stdout(Stdio::inherit());
            let status = process.spawn()?.wait()?;
            Ok(CommandOutcome {
                success: status.success(),
                code: status.code(),
                stdout: Vec::new(),
            })
        }
    }
}

#[cfg(test)]
#[path = "_tests_/command_tests.rs"]
mod command_tests;

/// Runs one command and maps process failures to the xtask error contract.
pub(crate) fn run(
    root: &Path,
    runner: &dyn CommandRunner,
    command: &CommandSpec,
) -> Result<CommandOutcome> {
    let outcome = match runner.execute(root, command) {
        Ok(outcome) => outcome,
        Err(source) => {
            return Err(Error::CommandStart {
                command: command.label(),
                source,
            });
        }
    };
    if outcome.success {
        Ok(outcome)
    } else {
        Err(Error::CommandFailed {
            command: command.label(),
            code: outcome.code,
        })
    }
}
