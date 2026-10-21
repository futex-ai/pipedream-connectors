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
available to callers. Configured values encode in the same untagged shape that
the API accepts. App references, SQL values, timer values, and property options
keep unknown object fields. Numeric values keep integer precision through the
JSON integer range.

Every paged response uses `Page<T>`. A missing page count remains unknown.
Only a zero count or a missing cursor stops `PageRequest::next`. Account
creation and retrieval return bare `Account` values.

Only credentials use redacted wrappers. Account creation mappings and nested
webhook signing keys stay redacted. Redirect and webhook URLs remain plain
strings.

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
- `src/accounts/` defines accounts, external users, and Connect token requests.
- `src/execution/` defines configuration, dynamic props, observations, and
  action results.
- `src/triggers/` defines emitters, deployed triggers, webhooks, workflows,
  and events.
- `src/admin/` defines projects, OAuth apps, overrides, webhooks, usage, and tokens.
- `src/wire.rs` defines data and error envelopes and response aliases.
- `tests/conformance.rs` checks nested response maps, union branches, nulls,
  optional fields, request bodies, and query models against vendored schemas.

### Related Docs

- [Connect resource protocol](../../docs/protocol/connect-resources.md)
- [Connect client protocol](../../docs/protocol/connect-client.md)
- [SDK architecture](../../docs/architecture.md)
- [SDK plan](../../plans/pipedream-connect-sdk.md)
