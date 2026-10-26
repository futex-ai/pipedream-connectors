//! Unit tests for the Rust file-length audit.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::command::{CommandOutcome, CommandRunnerMock, CommandSpec};
use crate::file_access::{DirectoryEntry, FileAccessMock};
use crate::file_length;

#[test]
fn accepts_a_rust_file_at_the_limit() {
    let path = PathBuf::from("crates/demo/src/lib.rs");
    let files = Unimock::new((
        FileAccessMock::read_dir
            .next_call(matching!(_))
            .returns(Ok(vec![DirectoryEntry {
                path: path.clone(),
                is_directory: false,
            }])),
        FileAccessMock::read_dir
            .next_call(matching!(_))
            .returns(Ok(Vec::new())),
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
        FileAccessMock::read_dir
            .next_call(matching!(_))
            .returns(Ok(Vec::new())),
        FileAccessMock::read_dir
            .next_call(matching!(_))
            .returns(Ok(vec![DirectoryEntry {
                path,
                is_directory: false,
            }])),
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

fn unused_runner() -> Unimock {
    Unimock::new(())
}
