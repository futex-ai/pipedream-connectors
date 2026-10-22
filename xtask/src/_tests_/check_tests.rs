//! Unit tests for the xtask command plan.

use crate::check;

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
