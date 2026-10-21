//! Runs the built binary without credentials to check help, error text, and exit codes.
//!
//! Each run clears the environment, so no command can reach Pipedream.

use std::process::{Command, Output};

/// Every implemented command, with the arguments that its grammar requires.
const COMMANDS: [&[&str]; 7] = [
    &["apps", "list"],
    &["apps", "get", "google_calendar"],
    &["categories", "list"],
    &["components", "list"],
    &["components", "get", "google_calendar-list-events"],
    &["actions", "list"],
    &[
        "actions",
        "run",
        "google_calendar-list-events",
        "--external-user-id",
        "user_1",
    ],
];

/// Settings that pass validation. Tests that use them fail before any request.
const SETTINGS: [&str; 8] = [
    "--client-id",
    "smoke-client-id",
    "--client-secret",
    "smoke-client-secret",
    "--project-id",
    "proj_Smoke1",
    "--environment",
    "development",
];

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_pipedream-cli"))
        .args(args)
        .env_clear()
        .output()
        .expect("the CLI binary runs")
}

fn text(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).expect("the CLI writes UTF-8 text")
}

#[test]
fn every_command_and_group_prints_its_help() {
    let groups: [&[&str]; 5] = [
        &[],
        &["apps"],
        &["categories"],
        &["components"],
        &["actions"],
    ];
    let paths = groups
        .iter()
        .map(|group| group.to_vec())
        .chain(COMMANDS.iter().map(|command| command[..2].to_vec()));
    for path in paths {
        let output = cli(&[&path[..], &["--help"]].concat());
        assert_eq!(output.status.code(), Some(0), "{path:?}");
        let usage = format!(
            "Usage: {}",
            ["pipedream-cli"]
                .iter()
                .chain(&path)
                .copied()
                .collect::<Vec<_>>()
                .join(" ")
        );
        let stdout = text(output.stdout);
        assert!(stdout.contains(&usage), "{path:?}: {stdout}");
        assert!(stdout.contains("PIPEDREAM_CLIENT_SECRET"), "{path:?}");
        assert!(output.stderr.is_empty(), "{path:?}");
    }
}

#[test]
fn every_command_without_credentials_exits_two_with_the_missing_setting() {
    for command in COMMANDS {
        let output = cli(command);
        assert_eq!(output.status.code(), Some(2), "{command:?}");
        assert!(output.stdout.is_empty(), "{command:?}");
        assert_eq!(
            text(output.stderr),
            "error: The client ID, client secret, project ID, and environment are missing. Set \
             PIPEDREAM_CLIENT_ID, PIPEDREAM_CLIENT_SECRET, PIPEDREAM_PROJECT_ID, and \
             PIPEDREAM_ENVIRONMENT, or pass their flags.\n",
            "{command:?}"
        );
    }
}

#[test]
fn a_missing_props_file_fails_before_any_request() {
    let command = [
        "actions",
        "run",
        "google_calendar-list-events",
        "--external-user-id",
        "user_1",
        "--props",
        "/nonexistent/pipedream-cli/props.json",
    ];
    let output = cli(&[&SETTINGS[..], &command].concat());
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert_eq!(
        text(output.stderr),
        "error: The --props file does not exist. Check the path.\n"
    );
}

#[test]
fn an_insecure_origin_fails_before_any_request() {
    let command = ["--api-origin", "http://localhost", "categories", "list"];
    let output = cli(&[&SETTINGS[..], &command].concat());
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        text(output.stderr),
        "error: --api-origin is not a valid API origin. Use an HTTPS origin without a path, \
         such as https://api.pipedream.com.\n"
    );
}

#[test]
fn version_and_usage_errors_use_their_exit_codes() {
    let version = cli(&["--version"]);
    assert_eq!(version.status.code(), Some(0));
    assert_eq!(
        text(version.stdout),
        format!("pipedream-cli {}\n", env!("CARGO_PKG_VERSION"))
    );
    let unknown = cli(&["apps", "remove"]);
    assert_eq!(unknown.status.code(), Some(2));
    assert!(text(unknown.stderr).starts_with("error: "));
}
