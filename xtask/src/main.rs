//! Workspace automation entry point.

mod check;
mod cli;
mod command;
mod error;
mod file_access;
mod file_length;
mod layout;
mod workspace;

use clap::Parser;

use crate::cli::{Cli, Command};
use crate::command::ProcessRunner;
use crate::error::Result;
use crate::file_access::OsFileAccess;
use crate::workspace::workspace_root;

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let cli = Cli::parse();
    let root = workspace_root()?;
    let command_runner = ProcessRunner;
    let file_access = OsFileAccess;
    match cli.command {
        Command::Check => check::run(&root, &command_runner, &file_access),
        Command::RustFileLengthLint { all } => {
            file_length::run(&root, all, &command_runner, &file_access)
        }
    }
}
