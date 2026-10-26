# pipedream-connect-types

`pipedream-connect-types` contains pure Rust models for the
Pipedream Connect API. Depend on this crate when an application needs typed
requests and responses without an HTTP client or an async runtime.

## Responsibilities

- Validate Connect identifiers and bounded request values.
- Serialize and deserialize registry, account, execution, trigger, and
  administration models.
- Decode date fields from RFC 3339 strings or epoch milliseconds and preserve
  the decoded wire form when models re-encode. Call `Timestamp::as_datetime`
  for the UTC value.
- Send trigger callback URLs unchanged when a client sends a deployment or
  webhook update.
- Preserve unknown response fields, enum values, and property definitions.
- Redact credential-class values in diagnostics while keeping wire values
  available.

## What This Crate Does

The crate defines serde models for the Connect REST API. It has no network,
clock, persistence, or async-runtime responsibilities. Response structs keep
unknown fields in flattened maps, and unknown enum or property values remain
available to callers. Configured values encode in the same untagged shape that
the API accepts. App references, SQL values, timer values, and property options
keep unknown object fields. JSON values keep integers exactly within the
`i64` and `u64` ranges. Larger integers and long decimals can round during
decoding. These models do not retain the original JSON number text.

Path identifiers reject the dot segments `.` and `..`. Pagination cursors are
opaque visible-ASCII values between 1 and 2048 bytes and may contain URL
punctuation.

Absent response fields stay omitted when their schemas do not permit null.
Nullable response fields keep explicit null output. Request deserialization
checks the page limit, trigger event limit, and Connect token lifetime.

Every paged response uses `Page<T>`. A missing page count remains unknown.
Only a zero count or a missing cursor stops `PageRequest::next`. Account
creation and retrieval return bare `Account` values.

`ListRegistryRequest` carries the filters shared by actions and registry
triggers. `ListComponentsRequest` also carries `component_type`.
`RunActionRequest` follows the Pipedream run-action schema and does not carry
a blocking flag. A run that returns 202 is always accepted asynchronously.

Trigger callback URLs are sent to Pipedream unchanged. The SDK does not inspect,
normalize, or reject their contents.

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
- `src/registry/` defines apps, components, enums, annotations, and list
  requests.
- `src/props/` defines lossless configurable props and configured values.
- `src/accounts/` defines accounts, account references, external users, and
  Connect token requests.
- `src/execution/` defines configuration, dynamic props, observations, and
  action results.
- `src/shared.rs` defines the lossless `Timestamp` used by date fields.
- `src/triggers/` defines emitters, deployed triggers, webhooks, workflows,
  and events.
- `src/admin/` defines projects, OAuth apps, overrides, webhooks, usage, and
  tokens.
- `src/wire.rs` defines data and error envelopes and response aliases.
- `tests/conformance/main.rs` checks nested response maps, union branches, nulls,
  optional fields, request bodies, and query models against vendored schemas.

### Related Docs

- [Connect resource protocol](../../docs/protocol/connect-resources.md)
- [Connect client protocol](../../docs/protocol/connect-client.md)
- [SDK architecture](../../docs/architecture.md)
- [SDK plan](../../plans/pipedream-connect-sdk.md)
