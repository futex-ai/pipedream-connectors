# pipedream-connect-types

`pipedream-connect-types` will provide the pure Rust models for the Pipedream
Connect API. Depend on this crate when an application needs API data types
without an HTTP client or an async runtime.

## Responsibilities

- Define typed identifiers, request models, response models, and wire envelopes.
- Preserve unknown API values so callers can handle new Pipedream data.
- Keep model validation and credential redaction independent of transport code.

## What This Crate Does

The crate is a model-only foundation for the SDK. It will contain serde
implementations for Connect resources, pagination, execution data, and
administration data. The bootstrap release has no public SDK models yet.

## Quick Start

Run the crate tests from the workspace:

```sh
cargo test -p pipedream-connect-types
```

## Development

Use the pinned toolchain and run the workspace gate before submitting changes:

```sh
cargo xtask check
```

### Key Code

- `src/lib.rs` is the crate root and will expose the model modules.
- The model modules will own validation and serde mappings as they are added.

### Related Docs

- [Connect resource protocol](../../docs/protocol/connect-resources.md)
- [Connect client protocol](../../docs/protocol/connect-client.md)
- [SDK architecture](../../docs/architecture.md)
