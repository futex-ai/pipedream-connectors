# xtask

`xtask` is the developer command runner for the Pipedream Connect workspace.
Run it from the repository root to execute the required local checks.

## Responsibilities

- Run the ordered Rust format, lint, test, documentation, file-length, and
  Rust module-layout checks.
- Report command output directly so failures remain actionable.
- Keep command and file access behind testable traits.

## What This Crate Does

The `check` command is the workspace gate. It does not implement SDK runtime
behaviour. It coordinates developer tools and local source audits. It returns
a typed error when a command cannot start or fails, or when an audit finds a
violation.

## Quick Start

Run the complete local gate from the repository root:

```sh
cargo xtask check
```

Run the Rust file-length audit for every workspace file:

```sh
cargo xtask rust-file-length-lint --all
```

## Development

Run focused xtask tests while changing the command runner or file audit:

```sh
cargo test -p xtask
```

### Key Code

- `src/main.rs` parses the task command and selects the workspace root.
- `src/check.rs` defines the ordered gate commands.
- `src/file_length.rs` audits Rust files against the 300-line cap.
- `src/layout.rs` rejects a Rust file beside a directory with the same module
  name.

### Related Docs

- [SDK architecture](../docs/architecture.md)
- [Project plan](../plans/)
