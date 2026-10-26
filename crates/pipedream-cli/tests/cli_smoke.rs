//! Runs the built binary without credentials to check help, error text, and exit codes.
//!
//! Each run clears the environment and points the API origin at a closed local
//! port, so no command can reach Pipedream.

use std::io::Write;
use std::process::{Command, Output, Stdio};

/// The API origin of every run: a local port that accepts no connection.
const ORIGIN: &str = "https://127.0.0.1:9";

/// One command: its path, the arguments that its grammar requires, and its
/// warning when a change may have happened without an answer.
type Case = (&'static str, &'static str, Option<&'static str>);

const RUN: &str = "The action may have run. Check its effects before you run it again.";
const TOKEN: &str = "Pipedream may have created the token. Run the command again for a new one.";
const WRITE: &str =
    "The upstream request may have run. Check its effects before you send it again.";
const DEPLOY: &str =
    "Pipedream may have deployed the trigger. Check triggers list before you deploy it again.";
const DELETE: &str = "Pipedream may have deleted the trigger. Check it with triggers get.";

/// The arguments of every proxy case.
const PROXY: &str = "/v1/items --external-user-id user_1 --account apn_Account1";

/// The arguments of every case that names a deployed trigger.
const TRIGGER: &str = "dc_Trigger1 --external-user-id user_1";

/// Every command that the CLI implements.
const CASES: [Case; 24] = [
    ("apps list", "", None),
    ("apps get", "google_calendar", None),
    ("categories list", "", None),
    ("components list", "", None),
    ("components get", "google_calendar-list-events", None),
    ("actions list", "", None),
    (
        "actions run",
        "google_calendar-list-events --external-user-id user_1",
        Some(RUN),
    ),
    ("tokens create", "--external-user-id user_1", Some(TOKEN)),
    ("accounts list", "", None),
    ("accounts get", "apn_Account1", None),
    ("users list", "", None),
    ("proxy get", PROXY, None),
    ("proxy post", PROXY, Some(WRITE)),
    ("proxy put", PROXY, Some(WRITE)),
    ("proxy patch", PROXY, Some(WRITE)),
    ("proxy delete", PROXY, Some(WRITE)),
    ("triggers list", "--external-user-id user_1", None),
    ("triggers get", TRIGGER, None),
    (
        "triggers deploy",
        "google_calendar-new-event --external-user-id user_1",
        Some(DEPLOY),
    ),
    ("triggers delete", TRIGGER, Some(DELETE)),
    ("triggers events", TRIGGER, None),
    ("project info", "", None),
    ("usage", "--start 1700000000 --end 1700086400", None),
    ("webhook get", "", None),
];

/// The command groups that have subcommands.
const GROUPS: [&str; 11] = [
    "apps",
    "categories",
    "components",
    "actions",
    "tokens",
    "accounts",
    "users",
    "proxy",
    "triggers",
    "project",
    "webhook",
];

/// Fake settings that pass validation. No request with them can reach Pipedream.
const VARIABLES: [(&str, &str); 4] = [
    ("PIPEDREAM_CLIENT_ID", "smoke-client-id"),
    ("PIPEDREAM_CLIENT_SECRET", "smoke-client-secret"),
    ("PIPEDREAM_PROJECT_ID", "proj_Smoke1"),
    ("PIPEDREAM_ENVIRONMENT", "development"),
];

fn words(text: &str) -> Vec<&str> {
    text.split_whitespace().collect()
}

fn line(case: &Case) -> Vec<&'static str> {
    [words(case.0), words(case.1)].concat()
}

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
    let paths = std::iter::once("")
        .chain(GROUPS)
        .chain(CASES.iter().map(|case| case.0));
    for path in paths {
        let output = cli(&[words(path), vec!["--help"]].concat(), &[], None);
        assert_eq!(output.status.code(), Some(0), "{path}");
        let stdout = text(output.stdout);
        let usage = format!("Usage: pipedream-cli {path}");
        assert!(stdout.contains(usage.trim_end()), "{path}: {stdout}");
        assert!(stdout.contains("PIPEDREAM_CLIENT_SECRET"), "{path}");
        assert!(output.stderr.is_empty(), "{path}");
    }
}

#[test]
fn every_command_without_credentials_exits_two_with_the_missing_settings() {
    for case in &CASES {
        let output = cli(&line(case), &[], None);
        assert_eq!(output.status.code(), Some(2), "{}", case.0);
        assert!(output.stdout.is_empty(), "{}", case.0);
        assert_eq!(
            text(output.stderr),
            "error: The client ID, client secret, project ID, and environment are missing. Set \
             PIPEDREAM_CLIENT_ID, PIPEDREAM_CLIENT_SECRET, PIPEDREAM_PROJECT_ID, and \
             PIPEDREAM_ENVIRONMENT.\n",
            "{}",
            case.0
        );
    }
}

#[test]
fn every_command_with_an_unreachable_origin_exits_six() {
    let note = format!("note: The CLI sends requests to {ORIGIN}, set by PIPEDREAM_API_ORIGIN.\n");
    for case in &CASES {
        let output = cli(&line(case), &VARIABLES, None);
        assert_eq!(output.status.code(), Some(6), "{}", case.0);
        assert!(output.stdout.is_empty(), "{}", case.0);
        let action = case
            .2
            .unwrap_or("Check the network and the API origin, then try again.");
        assert_eq!(
            text(output.stderr),
            format!("{note}error: The CLI could not reach Pipedream at {ORIGIN}. {action}\n"),
            "{}",
            case.0
        );
    }
}

#[test]
fn bad_props_json_on_standard_input_fails_before_any_request() {
    let command = words("actions run google_calendar-list-events --external-user-id user_1");
    let output = cli(
        &[command, vec!["--props", "-"]].concat(),
        &VARIABLES,
        Some(br#"{"a": "#),
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(text(output.stderr).ends_with(
        "error: --props is not valid JSON at line 1, column 6. Fix the JSON, then run the \
         command again.\n"
    ));
}

#[test]
fn a_missing_props_file_fails_before_any_request() {
    let command = words(
        "actions run google_calendar-list-events --external-user-id user_1 --props \
         /nonexistent/pipedream-cli/props.json",
    );
    let output = cli(&command, &VARIABLES, None);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        text(output.stderr).ends_with("error: The --props file does not exist. Check the path.\n")
    );
}

#[test]
fn an_insecure_or_padded_origin_fails_before_any_request() {
    for origin in ["http://localhost", "https://api.pipedream.com "] {
        let output = cli(
            &["--api-origin", origin, "categories", "list"],
            &VARIABLES,
            None,
        );
        assert_eq!(output.status.code(), Some(2), "{origin:?}");
        assert_eq!(
            text(output.stderr),
            "error: --api-origin is not a valid API origin. Use an HTTPS origin that has only a \
             host and an optional port, without surrounding spaces, such as \
             https://api.pipedream.com.\n",
            "{origin:?}"
        );
    }
}

#[test]
fn an_invalid_proxy_target_fails_before_any_request() {
    for target in [
        "/api\\items",
        "https://user@example.com/items",
        " /api/items",
    ] {
        let args = [
            "proxy",
            "get",
            target,
            "--external-user-id",
            "user_1",
            "--account",
            "apn_Account1",
        ];
        let output = cli(&args, &VARIABLES, None);
        assert_eq!(output.status.code(), Some(2), "{target}");
        assert!(output.stdout.is_empty(), "{target}");
        assert!(
            text(output.stderr).ends_with(
                "error: <URL> must be an HTTPS URL with a host, or a path that starts with one \
                 /. Remove any surrounding spaces, control characters, backslashes, credentials, \
                 and fragment.\n"
            ),
            "{target}"
        );
    }
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
