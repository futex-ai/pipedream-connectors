//! Unit tests for xtask workspace root discovery.

use std::path::PathBuf;

use crate::workspace::workspace_root_from;

#[test]
fn workspace_root_uses_manifest_parent() {
    assert_eq!(
        workspace_root_from(Some("/repo/xtask".into())).unwrap(),
        PathBuf::from("/repo")
    );
    assert!(workspace_root_from(None).is_err());
}
