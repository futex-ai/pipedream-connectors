//! Runs the built binary without credentials to check help, error text, and exit codes.
//!
//! Each run clears the environment and points the API origin at a closed local
//! port, so no command can reach Pipedream.

use std::io::Write;
use std::process::{Command, Output, Stdio};

/// The API origin of every run: a local port that accepts no connection.
const ORIGIN: &str = "https://127.0.0.1:9";

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

/// Commands that change data, so an unanswered request may have run.
const MUTATIONS: [&str; 1] = ["actions run"];

/// Fake settings that pass validation. No request with them can reach Pipedream.
const VARIABLES: [(&str, &str); 4] = [
    ("PIPEDREAM_CLIENT_ID", "smoke-client-id"),
    ("PIPEDREAM_CLIENT_SECRET", "smoke-client-secret"),
    ("PIPEDREAM_PROJECT_ID", "proj_Smoke1"),
    ("PIPEDREAM_ENVIRONMENT", "development"),
];

fn cli(args: &[&str], variables: &[(&str, &str)], stdin: Option<&[u8]>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_pipedream-cli"));
    command
        .args(args)
        .env_clear()
        .env("PIPEDREAM_API_ORIGIN", ORIGIN)
        .envs(variables.iter().copied())
        .stdin(match stdin {
            Some(_) => Stdio::piped(),
            None => Stdio::null(),
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn().expect("the CLI binary starts");
    if let Some(bytes) = stdin {
        let mut input = child.stdin.take().expect("the CLI has a standard input");
        input
            .write_all(bytes)
            .expect("the CLI reads standard input");
    }
    child.wait_with_output().expect("the CLI binary finishes")
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
        let output = cli(&[&path[..], &["--help"]].concat(), &[], None);
        assert_eq!(output.status.code(), Some(0), "{path:?}");
        let usage = format!("Usage: pipedream-cli {}", path.join(" "));
        let stdout = text(output.stdout);
        assert!(stdout.contains(usage.trim_end()), "{path:?}: {stdout}");
        assert!(stdout.contains("PIPEDREAM_CLIENT_SECRET"), "{path:?}");
        assert!(output.stderr.is_empty(), "{path:?}");
    }
}

#[test]
fn every_command_without_credentials_exits_two_with_the_missing_settings() {
    for command in COMMANDS {
        let output = cli(command, &[], None);
        assert_eq!(output.status.code(), Some(2), "{command:?}");
        assert!(output.stdout.is_empty(), "{command:?}");
        assert_eq!(
            text(output.stderr),
            "error: The client ID, client secret, project ID, and environment are missing. Set \
             PIPEDREAM_CLIENT_ID, PIPEDREAM_CLIENT_SECRET, PIPEDREAM_PROJECT_ID, and \
             PIPEDREAM_ENVIRONMENT.\n",
            "{command:?}"
        );
    }
}

#[test]
fn every_command_with_an_unreachable_origin_exits_six() {
    let note = format!("note: The CLI sends requests to {ORIGIN}, set by PIPEDREAM_API_ORIGIN.\n");
    for command in COMMANDS {
        let output = cli(command, &VARIABLES, None);
        assert_eq!(output.status.code(), Some(6), "{command:?}");
        assert!(output.stdout.is_empty(), "{command:?}");
        let action = if MUTATIONS.contains(&command[..2].join(" ").as_str()) {
            "The action may have run. Check its effects before you run it again."
        } else {
            "Check the network and the API origin, then try again."
        };
        assert_eq!(
            text(output.stderr),
            format!("{note}error: The CLI could not reach Pipedream at {ORIGIN}. {action}\n"),
            "{command:?}"
        );
    }
}

#[test]
fn bad_props_json_on_standard_input_fails_before_any_request() {
    let command = [
        "actions",
        "run",
        "google_calendar-list-events",
        "--external-user-id",
        "user_1",
        "--props",
        "-",
    ];
    let output = cli(&command, &VARIABLES, Some(br#"{"calendarId": "#));
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(text(output.stderr).ends_with(
        "error: --props is not valid JSON at line 1, column 15. Fix the JSON, then run the \
         command again.\n"
    ));
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
    let output = cli(&command, &VARIABLES, None);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        text(output.stderr).ends_with("error: The --props file does not exist. Check the path.\n")
    );
}

#[test]
fn an_insecure_origin_fails_before_any_request() {
    let output = cli(
        &["--api-origin", "http://localhost", "categories", "list"],
        &VARIABLES,
        None,
    );
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        text(output.stderr),
        "error: --api-origin is not a valid API origin. Use an HTTPS origin without a path, \
         such as https://api.pipedream.com.\n"
    );
}

#[test]
fn version_and_usage_errors_use_their_exit_codes() {
    let version = cli(&["--version"], &[], None);
    assert_eq!(version.status.code(), Some(0));
    assert_eq!(
        text(version.stdout),
        format!("pipedream-cli {}\n", env!("CARGO_PKG_VERSION"))
    );
    let unknown = cli(&["apps", "remove"], &[], None);
    assert_eq!(unknown.status.code(), Some(2));
    assert!(text(unknown.stderr).starts_with("error: "));
}
