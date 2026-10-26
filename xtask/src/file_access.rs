//! Injectable file access for the Rust file-length audit.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Boundary for filesystem reads used by xtask.
#[cfg_attr(test, unimock::unimock(api = FileAccessMock))]
pub trait FileAccess {
    /// Recursively finds Rust files below the workspace roots.
    fn rust_files(&self, root: &Path) -> io::Result<Vec<PathBuf>>;
    /// Reads one source file as UTF-8 text.
    fn read_to_string(&self, path: &Path) -> io::Result<String>;
    /// Finds Rust files beside directories with the same module name.
    fn rust_layout_violations(&self, root: &Path) -> io::Result<Vec<(PathBuf, PathBuf)>>;
}

/// Production filesystem implementation.
pub(crate) struct OsFileAccess;

impl FileAccess for OsFileAccess {
    fn rust_files(&self, root: &Path) -> io::Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        for relative_root in ["crates", "xtask"] {
            let path = root.join(relative_root);
            if path.exists() {
                collect_rust_files(&path, &mut files)?;
            }
        }
        files.sort();
        Ok(files)
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    fn rust_layout_violations(&self, root: &Path) -> io::Result<Vec<(PathBuf, PathBuf)>> {
        let mut violations = Vec::new();
        for relative_root in ["crates", "xtask"] {
            let path = root.join(relative_root);
            if path.exists() {
                collect_rust_layout_violations(&path, &mut violations)?;
            }
        }
        violations.sort();
        Ok(violations)
    }
}

fn collect_rust_files(path: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            collect_rust_files(&entry_path, files)?;
        } else if entry_path
            .extension()
            .is_some_and(|extension| extension == "rs")
        {
            files.push(entry_path);
        }
    }
    Ok(())
}

fn collect_rust_layout_violations(
    path: &Path,
    violations: &mut Vec<(PathBuf, PathBuf)>,
) -> io::Result<()> {
    let mut rust_files = BTreeMap::new();
    let mut directories = BTreeMap::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            directories.insert(entry.file_name(), entry_path);
        } else if entry_path
            .extension()
            .is_some_and(|extension| extension == "rs")
            && let Some(stem) = entry_path.file_stem()
        {
            rust_files.insert(stem.to_os_string(), entry_path);
        }
    }
    for (stem, file) in rust_files {
        if let Some(directory) = directories.get(&stem) {
            violations.push((file, directory.clone()));
        }
    }
    for directory in directories.into_values() {
        collect_rust_layout_violations(&directory, violations)?;
    }
    Ok(())
}
