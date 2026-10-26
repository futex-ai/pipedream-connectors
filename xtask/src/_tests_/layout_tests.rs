//! Unit tests for the Rust module-layout audit.

use std::path::{Path, PathBuf};

use unimock::{MockFn, Unimock, matching};

use crate::file_access::FileAccessMock;
use crate::layout;

#[test]
fn accepts_a_workspace_without_module_layout_conflicts() {
    let files = Unimock::new(
        FileAccessMock::rust_layout_violations
            .next_call(matching!(_))
            .returns(Ok(Vec::new())),
    );

    layout::run(Path::new("/workspace"), &files).unwrap();
}

#[test]
fn reports_a_file_beside_a_directory_with_the_same_name() {
    let files = Unimock::new(
        FileAccessMock::rust_layout_violations
            .next_call(matching!(_))
            .returns(Ok(vec![(
                PathBuf::from("/workspace/crates/demo/src/feature.rs"),
                PathBuf::from("/workspace/crates/demo/src/feature"),
            )])),
    );

    let error = layout::run(Path::new("/workspace"), &files)
        .unwrap_err()
        .to_string();

    assert!(error.contains("crates/demo/src/feature.rs"));
    assert!(error.contains("crates/demo/src/feature"));
}
