//! Injectable directory and file access for workspace audits.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// One entry returned by a directory listing.
#[derive(Debug, Clone)]
pub(crate) struct DirectoryEntry {
    /// The entry path.
    pub(crate) path: PathBuf,
    /// Whether the entry is a directory.
    pub(crate) is_directory: bool,
}

/// Boundary for filesystem reads used by xtask.
#[cfg_attr(
    test,
    unimock::unimock(api = [FileAccessReadDirMock, FileAccessReadToStringMock])
)]
pub(crate) trait FileAccess {
    /// Lists the entries in one directory.
    fn read_dir(&self, path: &Path) -> io::Result<Vec<DirectoryEntry>>;
    /// Reads one source file as UTF-8 text.
    fn read_to_string(&self, path: &Path) -> io::Result<String>;
}

/// Walks the workspace source roots and visits every directory listing.
pub(crate) fn walk_workspace_roots(
    root: &Path,
    file_access: &dyn FileAccess,
    mut visit: impl FnMut(&[DirectoryEntry]) -> io::Result<()>,
) -> io::Result<()> {
    for relative_root in ["crates", "xtask"] {
        let path = root.join(relative_root);
        let entries = match file_access.read_dir(&path) {
            Ok(entries) => entries,
            Err(source) if source.kind() == io::ErrorKind::NotFound => continue,
            Err(source) => return Err(source),
        };
        walk_directory(entries, file_access, &mut visit)?;
    }
    Ok(())
}

fn walk_directory(
    entries: Vec<DirectoryEntry>,
    file_access: &dyn FileAccess,
    visit: &mut impl FnMut(&[DirectoryEntry]) -> io::Result<()>,
) -> io::Result<()> {
    visit(&entries)?;
    for entry in entries {
        if entry.is_directory {
            let entries = file_access.read_dir(&entry.path)?;
            walk_directory(entries, file_access, visit)?;
        }
    }
    Ok(())
}

/// Production filesystem implementation.
pub(crate) struct OsFileAccess;

impl FileAccess for OsFileAccess {
    fn read_dir(&self, path: &Path) -> io::Result<Vec<DirectoryEntry>> {
        fs::read_dir(path)?
            .map(|entry| {
                let entry = entry?;
                let path = entry.path();
                Ok(DirectoryEntry {
                    is_directory: path.is_dir(),
                    path,
                })
            })
            .collect()
    }

    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }
}
