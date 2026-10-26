//! Integration check for the schema refresh script without a network request.

#![cfg(unix)]
#![allow(clippy::disallowed_methods)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

#[test]
fn refresh_script_targets_its_own_fixture_from_any_directory() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest.parent().unwrap().parent().unwrap();
    let scratch = workspace
        .join(".context")
        .join(format!("fixture-refresh-{}", std::process::id()));
    fs::create_dir_all(&scratch).unwrap();
    let fake_curl = scratch.join("curl");
    fs::write(
        &fake_curl,
        br#"#!/usr/bin/env bash
while (( $# )); do
    if [[ "$1" == "--output" ]]; then
        printf '%s\n' "$2"
        exit 0
    fi
    shift
done
exit 2
"#,
    )
    .unwrap();
    fs::set_permissions(&fake_curl, fs::Permissions::from_mode(0o755)).unwrap();
    let path = std::env::join_paths(
        std::iter::once(scratch.clone())
            .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    let output = Command::new("bash")
        .arg(manifest.join("tests/fixtures/refresh-openapi.sh"))
        .current_dir(&scratch)
        .env("PATH", path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let target = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        target.trim(),
        manifest
            .join("tests/fixtures/pipedream_openapi_swagger.json")
            .to_str()
            .unwrap()
    );
    fs::remove_dir_all(scratch).unwrap();
}
