//! Unit tests for the xtask command plan and filesystem audit.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use clap::Parser;
use unimock::{MockFn, Unimock, matching};

use crate::check;
use crate::cli::{Cli, Command};
use crate::command::{CommandOutcome, CommandRunnerMock, CommandSpec};
use crate::file_access::FileAccessMock;
use crate::file_length;
use crate::workspace::workspace_root_from;

#[test]
fn parses_supported_commands() {
    assert_eq!(
        Cli::try_parse_from(["xtask", "check"]).unwrap().command,
        Command::Check
    );
    assert_eq!(
        Cli::try_parse_from(["xtask", "rust-file-length-lint", "--all"])
            .unwrap()
            .command,
        Command::RustFileLengthLint { all: true }
    );
}

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
fn accepts_a_rust_file_at_the_limit() {
    let path = PathBuf::from("crates/demo/src/lib.rs");
    let files = Unimock::new((
        FileAccessMock::rust_files
            .next_call(matching!(_))
            .returns(Ok(vec![path.clone()])),
        FileAccessMock::read_to_string
            .next_call(matching!(_))
            .returns(Ok("line\n".repeat(300))),
    ));

    file_length::run(Path::new("/workspace"), true, &unused_runner(), &files).unwrap();
}

#[test]
fn reports_a_rust_file_over_the_limit() {
    let path = PathBuf::from("xtask/src/main.rs");
    let files = Unimock::new((
        FileAccessMock::rust_files
            .next_call(matching!(_))
            .returns(Ok(vec![path])),
        FileAccessMock::read_to_string
            .next_call(matching!(_))
            .returns(Ok("line\n".repeat(301))),
    ));

    let error = file_length::run(Path::new("/workspace"), true, &unused_runner(), &files)
        .unwrap_err()
        .to_string();
    assert!(error.contains("xtask/src/main.rs has 301 lines"));
}

#[test]
fn changed_file_audit_reads_both_git_lists() {
    let calls = Arc::new(Mutex::new(Vec::<CommandSpec>::new()));
    let calls_for_mock = Arc::clone(&calls);
    let runner = Unimock::new(
        CommandRunnerMock::execute
            .each_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, _, command| {
                calls_for_mock.lock().unwrap().push(command.clone());
                Ok(CommandOutcome {
                    success: true,
                    code: Some(0),
                    stdout: if calls_for_mock.lock().unwrap().len() == 1 {
                        b"crates/demo/src/lib.rs\0".to_vec()
                    } else {
                        Vec::new()
                    },
                })
            })),
    );
    let files = Unimock::new(
        FileAccessMock::read_to_string
            .next_call(matching!(_))
            .returns(Ok("line\n".repeat(2))),
    );

    file_length::run(Path::new("/workspace"), false, &runner, &files).unwrap();
    let calls = calls.lock().unwrap();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].program, "git");
    assert_eq!(calls[1].program, "git");
    assert!(calls.iter().all(|command| command.capture_output));
}

#[test]
fn workspace_root_uses_manifest_parent() {
    assert_eq!(
        workspace_root_from(Some("/repo/xtask".into())).unwrap(),
        PathBuf::from("/repo")
    );
    assert!(workspace_root_from(None).is_err());
}

fn unused_runner() -> Unimock {
    Unimock::new(())
}
