# pipedream-connect

`pipedream-connect` will provide the async server-side client for Pipedream
Connect. Depend on it when an application needs authenticated API requests,
resource operations, or the Pipedream API proxy.

## Responsibilities

- Authenticate with OAuth client credentials or a caller-provided bearer.
- Dispatch typed resource requests through an injectable HTTP transport.
- Enforce request limits, response bounds, retry rules, and error mapping.

## What This Crate Does

The crate will build on `pipedream-connect-types` and will offer resource
handles for the Connect REST API. Its reqwest transport will remain optional
so hosts can provide a different HTTP implementation. The bootstrap release
contains only the crate boundary and no client operations yet.

## Quick Start

Build and test the client crate from the workspace:

```sh
cargo test -p pipedream-connect --all-features
```

## Development

Use the pinned toolchain and run the workspace gate before submitting changes:

```sh
cargo xtask check
```

### Key Code

- `src/lib.rs` is the crate root and will expose client modules.
- The transport and resource modules will own side-effecting API behaviour.

### Related Docs

- [Connect client protocol](../../docs/protocol/connect-client.md)
- [Connect resource protocol](../../docs/protocol/connect-resources.md)
- [SDK architecture](../../docs/architecture.md)
