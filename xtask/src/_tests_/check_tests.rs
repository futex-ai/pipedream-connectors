//! Unit tests for the xtask command plan.

use std::path::Path;
use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::check;
use crate::command::{CommandOutcome, CommandRunnerMock, CommandSpec};
use crate::error::Error;
use crate::file_access::FileAccessMock;

#[test]
fn check_plan_has_the_required_order_and_flags() {
    let commands = check::commands();

    assert_eq!(commands.len(), 4);
    assert_eq!(commands[0].label(), "cargo fmt --all -- --check");
    assert!(!commands[0].capture_output);
    assert!(commands[1].args.contains(&"--locked".to_owned()));
    assert_eq!(commands[2].args[0], "test");
    assert_eq!(commands[3].args[0], "doc");
    assert_eq!(
        commands[3].environment,
        vec![("RUSTDOCFLAGS".to_owned(), "-D warnings".to_owned())]
    );
}

#[test]
fn failed_command_stops_the_gate_before_file_audits() {
    let calls = Arc::new(Mutex::new(Vec::<CommandSpec>::new()));
    let calls_for_mock = Arc::clone(&calls);
    let runner = Unimock::new(
        CommandRunnerMock::execute
            .each_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, _, command| {
                let mut calls = calls_for_mock.lock().unwrap();
                calls.push(command.clone());
                Ok(CommandOutcome {
                    success: calls.len() == 1,
                    code: if calls.len() == 1 { Some(0) } else { Some(17) },
                    stdout: Vec::new(),
                })
            })),
    );
    let files = Unimock::new(());

    let error = check::run(Path::new("/workspace"), &runner, &files).unwrap_err();

    assert!(matches!(
        error,
        Error::CommandFailed {
            command,
            code: Some(17)
        } if command == "cargo clippy --locked --workspace --all-targets --all-features -- -D warnings"
    ));
    assert_eq!(*calls.lock().unwrap(), check::commands()[..2].to_vec());
}

#[test]
fn clean_gate_runs_every_command_then_file_audits() {
    let calls = Arc::new(Mutex::new(Vec::<CommandSpec>::new()));
    let events = Arc::new(Mutex::new(Vec::<String>::new()));
    let calls_for_mock = Arc::clone(&calls);
    let events_for_runner = Arc::clone(&events);
    let runner = Unimock::new(
        CommandRunnerMock::execute
            .each_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, _, command| {
                calls_for_mock.lock().unwrap().push(command.clone());
                events_for_runner
                    .lock()
                    .unwrap()
                    .push(format!("command: {}", command.label()));
                Ok(CommandOutcome {
                    success: true,
                    code: Some(0),
                    stdout: Vec::new(),
                })
            })),
    );
    let events_for_files = Arc::clone(&events);
    let events_for_layout = Arc::clone(&events);
    let files = Unimock::new((
        FileAccessMock::rust_files
            .next_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                events_for_files
                    .lock()
                    .unwrap()
                    .push("file-length".to_owned());
                Ok(Vec::new())
            })),
        FileAccessMock::rust_layout_violations
            .next_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                events_for_layout.lock().unwrap().push("layout".to_owned());
                Ok(Vec::new())
            })),
    ));

    check::run(Path::new("/workspace"), &runner, &files).unwrap();

    assert_eq!(*calls.lock().unwrap(), check::commands());
    let expected_events = check::commands()
        .into_iter()
        .map(|command| format!("command: {}", command.label()))
        .chain(["file-length".to_owned(), "layout".to_owned()])
        .collect::<Vec<_>>();
    assert_eq!(*events.lock().unwrap(), expected_events);
}
