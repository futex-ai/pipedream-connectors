//! Error contract for workspace automation.

use std::fmt;
use std::io;
use std::path::PathBuf;

use thiserror::Error;

/// Result type used by xtask modules.
pub(crate) type Result<T> = std::result::Result<T, Error>;

/// One Rust file that exceeds the workspace line limit.
#[derive(Debug)]
pub(crate) struct FileLengthViolation {
    /// The source file path.
    pub(crate) path: PathBuf,
    /// The number of lines in the source file.
    pub(crate) lines: usize,
}

impl fmt::Display for FileLengthViolation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} has {} lines, allowed maximum is 300",
            self.path.display(),
            self.lines
        )
    }
}

/// A displayable list of Rust file-length violations.
#[derive(Debug)]
pub(crate) struct FileLengthViolations(pub(crate) Vec<FileLengthViolation>);

impl fmt::Display for FileLengthViolations {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut violations = self.0.iter();
        if let Some(first) = violations.next() {
            write!(formatter, "{first}")?;
            for violation in violations {
                write!(formatter, "\n{violation}")?;
            }
        }
        Ok(())
    }
}

/// One Rust file beside a directory with the same module name.
#[derive(Debug)]
pub(crate) struct ModuleLayoutViolation {
    /// The Rust file path.
    pub(crate) file: PathBuf,
    /// The adjacent directory path.
    pub(crate) directory: PathBuf,
}

impl fmt::Display for ModuleLayoutViolation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} sits beside module directory {}",
            self.file.display(),
            self.directory.display()
        )
    }
}

/// A displayable list of Rust module-layout violations.
#[derive(Debug)]
pub(crate) struct ModuleLayoutViolations(pub(crate) Vec<ModuleLayoutViolation>);

impl fmt::Display for ModuleLayoutViolations {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut violations = self.0.iter();
        if let Some(first) = violations.next() {
            write!(formatter, "{first}")?;
            for violation in violations {
                write!(formatter, "\n{violation}")?;
            }
        }
        Ok(())
    }
}

/// Failures reported by workspace automation.
#[derive(Debug, Error)]
pub(crate) enum Error {
    /// The task cannot locate its workspace manifest directory.
    #[error("[xtask/workspace] CARGO_MANIFEST_DIR is not set")]
    MissingManifestDir,
    /// The task manifest has no parent workspace directory.
    #[error("[xtask/workspace] manifest directory has no workspace parent: {manifest_dir:?}")]
    MissingWorkspaceParent { manifest_dir: PathBuf },
    /// A process could not start.
    #[error("[xtask/command] failed to start `{command}`: {source}")]
    CommandStart { command: String, source: io::Error },
    /// A process exited unsuccessfully.
    #[error("[xtask/command] `{command}` failed with exit code {code:?}")]
    CommandFailed { command: String, code: Option<i32> },
    /// A directory could not be read.
    #[error("[xtask/file_length] failed to read directory `{path:?}`: {source}")]
    ReadDirectory { path: PathBuf, source: io::Error },
    /// A Rust source file could not be read.
    #[error("[xtask/file_length] failed to read Rust file `{path:?}`: {source}")]
    ReadFile { path: PathBuf, source: io::Error },
    /// Git returned an invalid path stream.
    #[error("[xtask/file_length] git returned a non-UTF-8 path for `{command}`")]
    NonUtf8Path { command: String },
    /// One or more Rust files exceeded the line limit.
    #[error("[xtask/file_length] found {count} violation(s):\n{details}")]
    FileLengthViolations {
        /// Number of violations.
        count: usize,
        /// Typed violation details.
        details: FileLengthViolations,
    },
    /// One or more Rust files conflict with adjacent module directories.
    #[error("[xtask/layout] found {count} violation(s):\n{details}")]
    ModuleLayoutViolations {
        /// Number of violations.
        count: usize,
        /// Typed violation details.
        details: ModuleLayoutViolations,
    },
}
