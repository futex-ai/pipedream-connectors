# A guide for agents

Pipedream Connectors is a Rust workspace for a typed, server-side Pipedream
Connect SDK. Library code lives under `crates/`, and developer tasks live in
`xtask/`.

This file holds rules that are specific to this repository. The always-on
rules and detailed procedures live in the shared skills from `futex-ai/skills`.
Each skill description says when to read it.

## Communication

- Write responses in Simplified Technical English.
- Assume the reader has no context about this workspace.
- Report every failed, skipped, or blocked check with its reason.
- Ask for all required decisions in one numbered message.
- End each task with a report of changes, checks, and open items.

## Commands

- Build: `cargo build --workspace --all-features`
- Targeted tests: `cargo test -p pipedream-connect-types`
- Early check: `cargo fmt --all -- --check` and `cargo clippy --workspace
  --all-targets --all-features -- -D warnings`
- Complete gate: `cargo xtask check`
- Format: `cargo fmt --all`

## Repository rules

- Use the pinned Rust 1.95.0 toolchain in `rust-toolchain.toml`.
- Keep Rust files under `crates/` and `xtask/` at or below 300 lines.
- Keep protocol Markdown under 250 lines, and keep it separate from code docs.
- Keep scratch output under `.context/`, which Git ignores.
- Credential-gated live tests must skip cleanly when Pipedream variables are absent.

## Review focus

- Check alignment with the Connect protocol and architecture documents.
- Check credential redaction, identifier validation, response bounds, and retries.
- Check lossless decoding and webhook signature handling.
- Check crate boundaries, dependency features, and file-length compliance.

## Status

- The SDK is under active development and is not in production.
- Breaking changes are acceptable when they improve correctness or architecture.
