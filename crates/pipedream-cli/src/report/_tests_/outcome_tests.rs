//! Tests that results go to standard output and diagnostics go to standard error.

use std::io::{self, Write};

use clap::Parser;
use pipedream_connect_types::{Cursor, PageRequest};
use serde_json::json;

use crate::cli::Cli;
use crate::document::{Document, Notice, Output};
use crate::error::Error;
use crate::report::exit::Exit;
use crate::report::outcome::{finish, usage};
use crate::settings::{DEFAULT_ORIGIN, Setting};
use crate::terminal::Terminal;

/// A standard output that the reader closed.
struct Closed;

impl Write for Closed {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::from(io::ErrorKind::BrokenPipe))
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::from(io::ErrorKind::BrokenPipe))
    }
}

fn output(notices: Vec<Notice>) -> Output {
    Output {
        document: Document::revealed(&json!({"data": []})).unwrap(),
        notices,
    }
}

fn written(outcome: crate::error::Result<Output>) -> (Exit, String, String) {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let exit = finish(
        outcome,
        DEFAULT_ORIGIN,
        &mut Terminal::new(&mut out, &mut err),
    );
    (
        exit,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

#[test]
fn results_go_to_stdout_and_notices_to_stderr() {
    let next = PageRequest {
        after: Some(Cursor::new("cursor_9").unwrap()),
        ..PageRequest::default()
    };
    let notices = vec![Notice::PageLimit { pages: 2, next }, Notice::StuckCursor];
    let (exit, stdout, stderr) = written(Ok(output(notices)));
    assert_eq!(exit, Exit::Success);
    assert_eq!(stdout, "{\n  \"data\": []\n}\n");
    assert_eq!(
        stderr,
        "note: The CLI stopped after 2 pages. To read the next page, run the command again \
         with --after 'cursor_9'.\n\
         note: Pipedream repeated a page cursor, so the CLI stopped. The result can contain \
         repeated records.\n"
    );
}

#[test]
fn failures_go_to_stderr_with_their_exit_status() {
    let error = Error::MissingSettings {
        settings: vec![Setting::ProjectId],
    };
    let (exit, stdout, stderr) = written(Err(error));
    assert_eq!(exit, Exit::Usage);
    assert!(stdout.is_empty());
    assert_eq!(
        stderr,
        "error: The project ID is missing. Set PIPEDREAM_PROJECT_ID or pass --project-id.\n"
    );
}

#[test]
fn closed_stdout_is_an_internal_failure() {
    let (mut out, mut err) = (Closed, Vec::new());
    let exit = finish(
        Ok(output(Vec::new())),
        DEFAULT_ORIGIN,
        &mut Terminal::new(&mut out, &mut err),
    );
    assert_eq!(exit, Exit::Internal);
    assert_eq!(
        String::from_utf8(err).unwrap(),
        "error: The CLI cannot write to standard output. Check that the output stream is \
         open.\n"
    );
}

#[test]
fn help_goes_to_stdout_and_usage_errors_go_to_stderr() {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let help = Cli::try_parse_from(["pipedream-cli", "--help"]).unwrap_err();
    assert_eq!(
        usage(&help, &mut Terminal::new(&mut out, &mut err)),
        Exit::Success
    );
    assert!(
        String::from_utf8(out)
            .unwrap()
            .contains("Usage: pipedream-cli")
    );
    assert!(err.is_empty());

    let (mut out, mut err) = (Vec::new(), Vec::new());
    let unknown = Cli::try_parse_from(["pipedream-cli", "apps", "remove"]).unwrap_err();
    assert_eq!(
        usage(&unknown, &mut Terminal::new(&mut out, &mut err)),
        Exit::Usage
    );
    assert!(out.is_empty());
    assert!(String::from_utf8(err).unwrap().starts_with("error: "));
}

#[test]
fn help_on_a_closed_stdout_is_an_internal_failure() {
    let (mut out, mut err) = (Closed, Vec::new());
    let help = Cli::try_parse_from(["pipedream-cli", "--version"]).unwrap_err();
    assert_eq!(
        usage(&help, &mut Terminal::new(&mut out, &mut err)),
        Exit::Internal
    );
    assert!(!err.is_empty());
}
