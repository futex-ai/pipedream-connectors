//! Unit tests for xtask command-line parsing.

use clap::Parser;

use crate::cli::{Cli, Command};

#[test]
fn parses_supported_commands() {
    assert_eq!(
        Cli::try_parse_from(["xtask", "check"]).unwrap().command,
        Command::Check
    );
    assert_eq!(
        Cli::try_parse_from(["xtask", "rust-file-length-lint", "--all"])
            .unwrap()
            .command,
        Command::RustFileLengthLint { all: true }
    );
}
