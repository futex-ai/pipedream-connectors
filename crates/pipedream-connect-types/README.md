# pipedream-connect-types

`pipedream-connect-types` contains the pure, lossless Rust models for the
Pipedream Connect API. Depend on this crate when an application needs typed
requests and responses without an HTTP client or an async runtime.

## Responsibilities

- Validate Connect identifiers and bounded request values.
- Serialize and deserialize registry, account, execution, trigger, and
  administration models.
- Preserve unknown response fields, enum values, and property definitions.
- Redact credential-class values in diagnostics while keeping wire values available.

## What This Crate Does

The crate defines serde models for the Connect REST API. It has no network,
clock, persistence, or async-runtime responsibilities. Response structs keep
unknown fields in flattened maps, and unknown enum or property values remain
available to callers.

## Quick Start

```rust
use pipedream_connect_types::{ExternalUserId, PageRequest};

let user = ExternalUserId::new("customer_42")?;
let page = PageRequest::with_limit(50)?;
# let _: (ExternalUserId, PageRequest) = (user, page);
# Ok::<(), pipedream_connect_types::Error>(())
```

Run the model and conformance tests with:

```sh
cargo test -p pipedream-connect-types
```

## Development

Use the pinned toolchain and run the workspace gate before submitting changes:

```sh
cargo xtask check
```

Refresh the vendored OpenAPI fixture when the upstream schema changes:

```sh
crates/pipedream-connect-types/tests/fixtures/refresh-openapi.sh
```

### Key Code

- `src/ids.rs` defines validated path identifiers.
- `src/secret.rs` defines redacted `SecretString`, `SecretJson`, and
  `ConnectToken`.
- `src/pagination.rs` defines bounded `PageRequest`, `PageInfo`, and `Page<T>`.
- `src/registry/` defines apps, components, enums, annotations, and list requests.
- `src/props/` defines lossless configurable props and configured values.
- `src/accounts.rs` defines accounts, external users, and Connect token requests.
- `src/execution/` defines configuration, dynamic props, observations, and
  action results.
- `src/triggers/` defines emitters, deployed triggers, webhooks, workflows,
  and events.
- `src/admin/` defines projects, OAuth apps, overrides, webhooks, usage, and tokens.
- `src/wire.rs` defines generic data, page, and error envelopes.
- `tests/conformance.rs` generates response samples from the vendored OpenAPI schemas.

### Related Docs

- [Connect resource protocol](../../docs/protocol/connect-resources.md)
- [Connect client protocol](../../docs/protocol/connect-client.md)
- [SDK architecture](../../docs/architecture.md)
- [SDK plan](../../plans/pipedream-connect-sdk.md)
