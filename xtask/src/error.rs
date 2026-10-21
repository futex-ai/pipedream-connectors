//! Error contract for workspace automation.

use std::io;
use std::path::PathBuf;

use thiserror::Error;

/// Result type used by xtask modules.
pub(crate) type Result<T> = std::result::Result<T, Error>;

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
    FileLengthViolations { count: usize, details: String },
}
