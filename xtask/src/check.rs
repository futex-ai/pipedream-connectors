//! The ordered credential-free workspace verification gate.

use std::path::Path;

use crate::command::{self, CommandRunner, CommandSpec};
use crate::error::Result;
use crate::file_access::FileAccess;
use crate::file_length;
use crate::layout;

/// Runs every required check in order.
pub(crate) fn run(
    root: &Path,
    command_runner: &dyn CommandRunner,
    file_access: &dyn FileAccess,
) -> Result<()> {
    for command in commands() {
        command::run(root, command_runner, &command)?;
    }
    file_length::run(root, true, command_runner, file_access)?;
    layout::run(root, file_access)
}

/// Returns the command plan for the workspace gate.
pub(crate) fn commands() -> Vec<CommandSpec> {
    vec![
        CommandSpec::inherited("cargo", &["fmt", "--all", "--", "--check"]),
        CommandSpec::inherited(
            "cargo",
            &[
                "clippy",
                "--locked",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--",
                "-D",
                "warnings",
            ],
        ),
        CommandSpec::inherited(
            "cargo",
            &["test", "--locked", "--workspace", "--all-features"],
        ),
        CommandSpec::inherited(
            "cargo",
            &[
                "doc",
                "--locked",
                "--workspace",
                "--no-deps",
                "--all-features",
            ],
        )
        .with_environment("RUSTDOCFLAGS", "-D warnings"),
    ]
}

#[cfg(test)]
#[path = "_tests_/check_tests.rs"]
mod check_tests;
