//! Rust module-layout audit for workspace code.

use std::path::{Path, PathBuf};

use crate::error::{Error, ModuleLayoutViolation, ModuleLayoutViolations, Result};
use crate::file_access::FileAccess;

/// Runs the module-layout audit for the workspace roots.
pub(crate) fn run(root: &Path, file_access: &dyn FileAccess) -> Result<()> {
    let violations = match file_access.rust_layout_violations(root) {
        Ok(violations) => violations,
        Err(source) => {
            return Err(Error::ReadDirectory {
                path: root.to_path_buf(),
                source,
            });
        }
    };
    if violations.is_empty() {
        return Ok(());
    }
    let details: Vec<ModuleLayoutViolation> = violations
        .into_iter()
        .map(|(file, directory)| ModuleLayoutViolation {
            file: display_path(root, &file),
            directory: display_path(root, &directory),
        })
        .collect();
    Err(Error::ModuleLayoutViolations {
        count: details.len(),
        details: ModuleLayoutViolations(details),
    })
}

fn display_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

#[cfg(test)]
#[path = "_tests_/layout_tests.rs"]
mod layout_tests;
