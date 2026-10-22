//! Workspace root discovery for xtask.

use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

use crate::error::{Error, Result};

/// Finds the workspace root from Cargo's manifest directory.
pub(crate) fn workspace_root() -> Result<PathBuf> {
    workspace_root_from(env::var_os("CARGO_MANIFEST_DIR"))
}

/// Finds the workspace root from an optional manifest directory.
pub(crate) fn workspace_root_from(manifest_dir: Option<OsString>) -> Result<PathBuf> {
    let manifest_dir = manifest_dir
        .map(PathBuf::from)
        .ok_or(Error::MissingManifestDir)?;
    manifest_dir
        .parent()
        .map(PathBuf::from)
        .ok_or(Error::MissingWorkspaceParent { manifest_dir })
}

#[cfg(test)]
#[path = "_tests_/workspace_tests.rs"]
mod workspace_tests;
