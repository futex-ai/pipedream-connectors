//! Rust source file-length audit for workspace code.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::command::{self, CommandRunner, CommandSpec};
use crate::error::{Error, FileLengthViolation, FileLengthViolations, Result};
use crate::file_access::FileAccess;

const MAX_LINES: usize = 300;

/// Runs the file-length audit for all or changed Rust files.
pub(crate) fn run(
    root: &Path,
    all: bool,
    command_runner: &dyn CommandRunner,
    file_access: &dyn FileAccess,
) -> Result<()> {
    let files = if all {
        match file_access.rust_files(root) {
            Ok(files) => files,
            Err(source) => {
                return Err(Error::ReadDirectory {
                    path: root.to_path_buf(),
                    source,
                });
            }
        }
    } else {
        changed_rust_files(root, command_runner)?
    };
    let mut violations = Vec::new();
    for path in files {
        let contents = match file_access.read_to_string(&path) {
            Ok(contents) => contents,
            Err(source) => {
                return Err(Error::ReadFile {
                    path: path.clone(),
                    source,
                });
            }
        };
        let line_count = contents.lines().count();
        if line_count > MAX_LINES {
            violations.push(FileLengthViolation {
                path: display_path(root, &path).into(),
                lines: line_count,
            });
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(Error::FileLengthViolations {
            count: violations.len(),
            details: FileLengthViolations(violations),
        })
    }
}

fn changed_rust_files(root: &Path, runner: &dyn CommandRunner) -> Result<Vec<PathBuf>> {
    let mut files = BTreeSet::new();
    for args in [
        &[
            "diff",
            "--name-only",
            "-z",
            "--diff-filter=ACMR",
            "origin/main",
            "--",
            "crates",
            "xtask",
        ][..],
        &[
            "ls-files",
            "-z",
            "--others",
            "--exclude-standard",
            "--",
            "crates",
            "xtask",
        ][..],
    ] {
        let command = CommandSpec::captured("git", args);
        let output = command::run(root, runner, &command)?;
        for relative in output.stdout.split(|byte| *byte == 0) {
            if relative.is_empty() {
                continue;
            }
            let relative = match std::str::from_utf8(relative) {
                Ok(relative) => relative,
                Err(_) => {
                    return Err(Error::NonUtf8Path {
                        command: command.label(),
                    });
                }
            };
            if relative.ends_with(".rs") {
                files.insert(root.join(relative));
            }
        }
    }
    Ok(files.into_iter().collect())
}

fn display_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}
