//! Command-line parsing for workspace checks.

use clap::{Parser, Subcommand};

/// Parsed xtask command.
#[derive(Debug, PartialEq, Eq, Parser)]
#[command(name = "xtask", about = "Pipedream Connect workspace tasks")]
pub(crate) struct Cli {
    /// Task selected by the caller.
    #[command(subcommand)]
    pub(crate) command: Command,
}

/// Supported developer tasks.
#[derive(Debug, PartialEq, Eq, Subcommand)]
pub(crate) enum Command {
    /// Run the complete workspace verification gate.
    Check,
    /// Audit Rust files against the 300-line limit.
    RustFileLengthLint {
        /// Scan every Rust file instead of changed files.
        #[arg(long)]
        all: bool,
    },
}

#[cfg(test)]
#[path = "_tests_/cli_tests.rs"]
mod cli_tests;
