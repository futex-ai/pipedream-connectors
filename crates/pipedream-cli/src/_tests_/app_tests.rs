//! Tests for the pipeline order: parse, settings, validation, connection, and report.

use std::io;

use pipedream_connect::error::{Error, InvalidRequest};
use unimock::{MockFn, matching};

use crate::connector::ConnectMock;
use crate::report::exit::Exit;
use crate::test_support::{Env, SECRET, run};

#[tokio::test]
async fn help_prints_to_stdout_without_reading_any_variable() {
    let ran = run(&["--help"], ()).await;
    assert_eq!(ran.exit, Exit::Success);
    assert!(
        ran.stdout
            .contains("Usage: pipedream-cli [OPTIONS] <COMMAND>")
    );
    assert!(ran.stderr.is_empty());
    let ran = run(&["actions", "run", "--help"], ()).await;
    assert_eq!(ran.exit, Exit::Success);
    assert!(ran.stdout.contains("--external-user-id <ID>"));
}

#[tokio::test]
async fn version_prints_to_stdout() {
    let ran = run(&["--version"], ()).await;
    assert_eq!(ran.exit, Exit::Success);
    assert_eq!(
        ran.stdout,
        format!("pipedream-cli {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[tokio::test]
async fn usage_errors_exit_two_without_reading_settings() {
    for args in [
        &["apps", "list", "--limit"][..],
        &["apps", "remove"],
        &["actions", "run", "google_calendar-list-events"],
        &["components", "list", "--after", "a", "--before", "b"],
        &["apps", "list", "--max-pages", "3"],
        &["apps", "list", "--all", "--max-pages", "0"],
        &[],
    ] {
        let ran = run(args, ()).await;
        assert_eq!(ran.exit, Exit::Usage, "{args:?}");
        assert!(ran.stdout.is_empty(), "{args:?}");
        assert!(!ran.stderr.is_empty(), "{args:?}");
    }
}

#[tokio::test]
async fn missing_settings_exit_two_before_any_connection() {
    let ran = run(&["categories", "list"], Env::EMPTY.clauses()).await;
    assert_eq!(ran.exit, Exit::Usage);
    assert!(ran.stdout.is_empty());
    assert_eq!(
        ran.stderr,
        "error: The client ID, client secret, project ID, and environment are missing. Set \
         PIPEDREAM_CLIENT_ID, PIPEDREAM_CLIENT_SECRET, PIPEDREAM_PROJECT_ID, and \
         PIPEDREAM_ENVIRONMENT, or pass their flags.\n"
    );
}

#[tokio::test]
async fn setting_errors_never_print_the_secret() {
    let env = Env {
        project_id: Some("not-a-project"),
        ..Env::CONFIGURED
    };
    let ran = run(&["categories", "list"], env.clauses()).await;
    assert_eq!(ran.exit, Exit::Usage);
    assert_eq!(
        ran.stderr,
        "error: PIPEDREAM_PROJECT_ID is not a valid project ID. Use the project ID that starts \
         with proj_.\n"
    );
    let ran = run(
        &["categories", "list", "--client-id", " padded"],
        Env::CONFIGURED.clauses(),
    )
    .await;
    assert_eq!(ran.exit, Exit::Usage);
    assert!(
        ran.stderr
            .contains("--client-id and PIPEDREAM_CLIENT_SECRET")
    );
    assert!(!ran.stderr.contains(SECRET));
    assert!(!ran.stdout.contains(SECRET));
}

#[tokio::test]
async fn the_connector_receives_the_resolved_settings() {
    let clauses = (
        Env::CONFIGURED.clauses(),
        ConnectMock
            .next_call(
                matching!((settings) if settings.project_id.as_str() == "proj_Flag1"
                && settings.api_origin.as_deref() == Some("https://api.example.com")),
            )
            .answers(&|_, _| Err(Error::internal(io::Error::other("tls backend")))),
    );
    let ran = run(
        &[
            "categories",
            "list",
            "--project-id",
            "proj_Flag1",
            "--api-origin",
            "https://api.example.com",
        ],
        clauses,
    )
    .await;
    assert_eq!(ran.exit, Exit::Internal);
    assert_eq!(
        ran.stderr,
        "error: The CLI could not start its HTTP client. Run the command again.\n"
    );
}

#[tokio::test]
async fn rejected_client_settings_exit_two() {
    let clauses = (
        Env::CONFIGURED.clauses(),
        ConnectMock
            .next_call(matching!(_))
            .answers(&|_, _| Err(Error::InvalidRequest(InvalidRequest::Origin))),
    );
    let ran = run(&["categories", "list"], clauses).await;
    assert_eq!(ran.exit, Exit::Usage);
    assert_eq!(
        ran.stderr,
        "error: The connection settings are not valid. Check the configuration flags and \
         variables.\n"
    );
}
