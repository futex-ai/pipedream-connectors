//! Unit tests for the Rust module-layout audit.

use std::io;
use std::path::{Path, PathBuf};

use unimock::{MockFn, Unimock, matching};

use crate::file_access::{DirectoryEntry, FileAccessReadDirMock};
use crate::layout;

#[test]
fn accepts_a_workspace_without_module_layout_conflicts() {
    let files = Unimock::new((
        FileAccessReadDirMock
            .next_call(matching!(_))
            .returns(Ok(Vec::new())),
        FileAccessReadDirMock
            .next_call(matching!(_))
            .returns(Ok(Vec::new())),
    ));

    layout::run(Path::new("/workspace"), &files).unwrap();
}

#[test]
fn reports_a_file_beside_a_directory_with_the_same_name() {
    let files = Unimock::new((
        FileAccessReadDirMock
            .next_call(matching!(_))
            .returns(Ok(vec![
                DirectoryEntry {
                    path: PathBuf::from("/workspace/crates/demo/src/feature.rs"),
                    is_directory: false,
                },
                DirectoryEntry {
                    path: PathBuf::from("/workspace/crates/demo/src/feature"),
                    is_directory: true,
                },
            ])),
        FileAccessReadDirMock
            .next_call(matching!(_))
            .returns(Ok(Vec::new())),
        FileAccessReadDirMock
            .next_call(matching!(_))
            .returns(Ok(Vec::new())),
    ));

    let error = layout::run(Path::new("/workspace"), &files)
        .unwrap_err()
        .to_string();

    assert!(error.contains("crates/demo/src/feature.rs"));
    assert!(error.contains("crates/demo/src/feature"));
}

#[test]
fn reports_layout_directory_read_errors_with_layout_prefix() {
    let files = Unimock::new(FileAccessReadDirMock.next_call(matching!(_)).returns(Err(
        io::Error::new(io::ErrorKind::PermissionDenied, "no access"),
    )));

    let error = layout::run(Path::new("/workspace"), &files)
        .unwrap_err()
        .to_string();

    assert!(error.starts_with("[xtask/layout] failed to read directory"));
}

#[test]
fn skips_a_missing_crates_root_and_reports_an_xtask_violation() {
    let files = Unimock::new((
        FileAccessReadDirMock
            .next_call(matching!(_))
            .returns(Err(io::Error::new(io::ErrorKind::NotFound, "missing"))),
        FileAccessReadDirMock
            .next_call(matching!(_))
            .returns(Ok(vec![
                DirectoryEntry {
                    path: PathBuf::from("/workspace/xtask/module.rs"),
                    is_directory: false,
                },
                DirectoryEntry {
                    path: PathBuf::from("/workspace/xtask/module"),
                    is_directory: true,
                },
            ])),
        FileAccessReadDirMock
            .next_call(matching!(_))
            .returns(Ok(Vec::new())),
    ));

    let error = layout::run(Path::new("/workspace"), &files)
        .unwrap_err()
        .to_string();

    assert!(error.contains("xtask/module.rs"));
    assert!(error.contains("xtask/module"));
}

#[test]
fn reports_a_nested_not_found_with_layout_prefix() {
    let files = Unimock::new((
        FileAccessReadDirMock
            .next_call(matching!(_))
            .returns(Ok(vec![DirectoryEntry {
                path: PathBuf::from("/workspace/crates/nested"),
                is_directory: true,
            }])),
        FileAccessReadDirMock
            .next_call(matching!(_))
            .returns(Err(io::Error::new(io::ErrorKind::NotFound, "gone"))),
    ));

    let error = layout::run(Path::new("/workspace"), &files)
        .unwrap_err()
        .to_string();

    assert!(error.starts_with("[xtask/layout] failed to read directory"));
}
