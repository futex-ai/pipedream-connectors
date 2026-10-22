//! Tests for `triggers list`, `get`, `delete`, and `events`.

mod delete_tests;
mod read_tests;

/// The arguments of `command` for the deployed trigger `dc_Trigger1`.
fn trigger(command: &str) -> Vec<&str> {
    vec![
        "triggers",
        command,
        "dc_Trigger1",
        "--external-user-id",
        "user_1",
    ]
}
