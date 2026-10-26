# A guide for agents

Pipedream Connectors is a Rust workspace for a typed, server-side Pipedream
Connect SDK. Library code lives under `crates/`, and developer tasks live in
`xtask/`.

This file holds rules that are specific to this repository. The always-on
rules and detailed procedures live in the shared skills from `futex-ai/skills`.
Each skill description says when to read it.

## Communication

- Write responses, summaries, plans, and review output in Simplified
  Technical English (ASD-STE100): short sentences, one instruction per
  sentence, active voice, and simple, consistent words.
- Assume the reader has no context. Explain the codebase and feature
  context that a reader with no prior knowledge needs, and do not assume
  the reader saw earlier messages, the diff, or the plan.
- Report outcomes faithfully. Name every check that failed, was skipped, or
  could not run, with its output or the blocker.
- Ask for all decisions in one numbered message. List decisions and review
  findings by severity, highest first. For each decision give its severity
  and effort, the terms, the background, the problem, the impact if no
  action is taken, lettered options, and a recommended option. Do the
  independent work first, then stop and wait.
- End every task with a final report: what changed and the commit, each
  decision and its outcome, the checks that ran, anything skipped, and the
  outstanding decisions or review items that still need the user.

## Commands

- Build: `cargo build --workspace --all-features`
- Targeted tests: `cargo test -p <crate> <test name filter>`
- Early check: `cargo fmt --all -- --check` and `cargo clippy --workspace
  --all-targets --all-features -- -D warnings`
- Complete gate: `cargo xtask check` (format, locked clippy, tests, docs,
  file-length, and Rust module-layout audits)
- Rust file-length lint: `cargo xtask rust-file-length-lint --all`
- CLI smoke without credentials: `cargo test -p pipedream-cli --test cli_smoke`
- Markdown lint: `npx --yes markdownlint-cli2 <files>`
- Format: `cargo fmt --all`

## Repository rules

- Use the pinned toolchain in `rust-toolchain.toml`.
- Keep Rust files under `crates/` and `xtask/` at or below 300 lines.
- Do not place `<name>.rs` beside a `<name>/` directory under `crates/` or
  `xtask/`.
- Credential-gated live tests must skip cleanly when Pipedream variables are absent.
- Live tests also require `PIPEDREAM_LIVE_TESTS=1`.

## Review focus

- Check alignment with the Connect protocol and architecture documents.
- Check credential redaction, identifier validation, response bounds, and retries.
- Check lossless decoding and webhook signature handling.
- Check crate boundaries, dependency features, and file-length compliance.

## Status

- The SDK is under active development and is not in production.
- Breaking changes are acceptable when they improve correctness or architecture.
