//! Injectable directory and file access for workspace audits.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// One entry returned by a directory listing.
#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    /// The entry path.
    pub(crate) path: PathBuf,
    /// Whether the entry is a directory.
    pub(crate) is_directory: bool,
}

/// Boundary for filesystem reads used by xtask.
#[cfg_attr(test, unimock::unimock(api = FileAccessMock))]
pub(crate) trait FileAccess {
    /// Lists the entries in one directory.
    fn read_dir(&self, path: &Path) -> io::Result<Vec<DirectoryEntry>>;
    /// Reads one source file as UTF-8 text.
    fn read_to_string(&self, path: &Path) -> io::Result<String>;
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
