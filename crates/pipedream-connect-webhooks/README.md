# pipedream-connect-webhooks

`pipedream-connect-webhooks` will verify Pipedream Connect webhook signatures
and model webhook events. Depend on it when a receiver needs webhook handling
without the outbound API client.

## Responsibilities

- Verify signed webhook bodies with a bounded, replay-resistant contract.
- Decode connection events and trigger deliveries into typed values.
- Keep webhook handling independent from outbound HTTP transport.

## What This Crate Does

The crate will use `pipedream-connect-types` for shared identifiers and
credential wrappers. It will support connection events and trigger deliveries
described by the webhook protocol. The bootstrap release contains only the
crate boundary and no verification APIs yet.

## Quick Start

Build and test the webhook crate from the workspace:

```sh
cargo test -p pipedream-connect-webhooks
```

## Development

Use the pinned toolchain and run the workspace gate before submitting changes:

```sh
cargo xtask check
```

### Key Code

- `src/lib.rs` is the crate root and will expose verification modules.
- The verifier and event modules will own webhook boundary behaviour.

### Related Docs

- [Webhook protocol](../../docs/protocol/webhooks.md)
- [Connect resource protocol](../../docs/protocol/connect-resources.md)
- [SDK architecture](../../docs/architecture.md)
