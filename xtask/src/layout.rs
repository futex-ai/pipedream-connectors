//! Rust module-layout audit for workspace code.

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};

use crate::error::{Error, ModuleLayoutViolation, ModuleLayoutViolations, Result};
use crate::file_access::FileAccess;

/// Runs the module-layout audit for the workspace roots.
pub(crate) fn run(root: &Path, file_access: &dyn FileAccess) -> Result<()> {
    let violations = match rust_layout_violations(root, file_access) {
        Ok(violations) => violations,
        Err(source) => {
            return Err(Error::ReadLayout {
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

fn rust_layout_violations(
    root: &Path,
    file_access: &dyn FileAccess,
) -> io::Result<Vec<(PathBuf, PathBuf)>> {
    let mut violations = Vec::new();
    for relative_root in ["crates", "xtask"] {
        let path = root.join(relative_root);
        match collect_rust_layout_violations(&path, file_access, &mut violations) {
            Ok(()) => {}
            Err(source) if source.kind() == io::ErrorKind::NotFound => {}
            Err(source) => return Err(source),
        }
    }
    violations.sort();
    Ok(violations)
}

fn collect_rust_layout_violations(
    path: &Path,
    file_access: &dyn FileAccess,
    violations: &mut Vec<(PathBuf, PathBuf)>,
) -> io::Result<()> {
    let mut rust_files = BTreeMap::new();
    let mut directories = BTreeMap::new();
    for entry in file_access.read_dir(path)? {
        if entry.is_directory {
            if let Some(file_name) = entry.path.file_name() {
                directories.insert(file_name.to_os_string(), entry.path);
            }
        } else if entry
            .path
            .extension()
            .is_some_and(|extension| extension == "rs")
            && let Some(stem) = entry.path.file_stem()
        {
            rust_files.insert(stem.to_os_string(), entry.path);
        }
    }
    for (stem, file) in rust_files {
        if let Some(directory) = directories.get(&stem) {
            violations.push((file, directory.clone()));
        }
    }
    for directory in directories.into_values() {
        collect_rust_layout_violations(&directory, file_access, violations)?;
    }
    Ok(())
}

fn display_path(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

#[cfg(test)]
#[path = "_tests_/layout_tests.rs"]
mod layout_tests;
