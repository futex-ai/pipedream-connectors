# pipedream-connect-webhooks

`pipedream-connect-webhooks` verifies Pipedream Connect webhook signatures and
models webhook events. Depend on it when a receiver needs webhook handling
without the outbound API client.

## Responsibilities

- Verify signed webhook bodies with a bounded, replay-resistant contract.
- Decode connection events and trigger deliveries into typed values.
- Keep webhook handling independent from outbound HTTP transport.

## What This Crate Does

The crate uses `pipedream-connect-types` for shared identifiers, account
models, environments, and credential wrappers. It verifies the exact raw body
with HMAC-SHA256 and a caller-supplied receipt time. The verifier defaults to a
10 MiB body limit. Use `with_body_limit` to change it. The crate decodes
connection events and trigger deliveries without an async runtime or HTTP
server. A delivery without a context ID uses lowercase hexadecimal SHA-256 of
the exact raw body as its stable event ID. `decode` uses the default 10 MiB
body limit. Use `decode_with_limit` to select another limit. Both methods hash
all raw body bytes.
Direct serde decoding hashes the JSON value text without outer whitespace.
Both decoding paths reject an invalid context ID.

Session identifiers and trigger event JSON use `RawJson`. Top-level unknown
fields in connection and trigger payloads, and unknown fields in trigger
context, also use `RawJson`. Call `as_str()` for the exact JSON text. Call
`decode::<T>()` for typed access.
The stored text preserves large integers and long decimals. Typed access uses
the selected type's number precision. `new` and `decode` return `PayloadError`
with the original serde error as their source when conversion fails.

Connection accounts use the shared models and redacted credential wrappers.
Those models use `serde_json::Value` for JSON fields. They keep integers exactly
within the `i64` and `u64` ranges. Larger integers and long decimals can round.
Only fields stored as `RawJson` retain exact JSON number text.

Trigger context timestamps use the shared `Timestamp` type. They accept RFC
3339 strings and epoch milliseconds. Call `Timestamp::as_datetime` for the UTC
value. An invalid timestamp remains in the unknown context fields so the
delivery can still be decoded.

`FaultClass::Sender` marks invalid headers, signatures, payloads, and body
sizes. These errors keep their 4xx status codes. `FaultClass::Host` marks an
empty signing key, an invalid receipt clock, or another host configuration
fault. These errors return status 500. `SignatureError::fault_class` and
`PayloadError::fault_class` expose the class used by `status_code`.

## Quick Start

Verify a raw body before decoding its delivery:

```rust
use std::time::{Duration, UNIX_EPOCH};

use pipedream_connect_types::SecretString;
use pipedream_connect_webhooks::{
    HmacWebhookSignatureVerifier, TriggerDelivery, WebhookSignatureVerifier,
};

let body = b"{\"event\":\"created\"}\n";
let verifier = HmacWebhookSignatureVerifier::new();
verifier.verify(
    &SecretString::new("webhook-test-key"),
    "t=1000,v1=6e77b54efa731c70ca5c7004572d6ac24203eebfd2dc3b4981b82de56d187ea5",
    body,
    UNIX_EPOCH + Duration::from_secs(1000),
)?;
let delivery = TriggerDelivery::decode(body)?;
assert!(!delivery.identity().is_empty());
# Ok::<(), Box<dyn std::error::Error>>(())
```

Run the crate tests from the workspace:

```sh
cargo test -p pipedream-connect-webhooks
```

## Development

Use the pinned toolchain and run the workspace gate before submitting changes:

```sh
cargo xtask check
```

### Key Code

- `src/signature.rs` defines `WebhookSignatureVerifier`,
  `HmacWebhookSignatureVerifier`, and `SignatureError`.
- `src/events.rs` defines `ConnectionEvent` and its lossless event models.
- `src/raw_json.rs` defines exact `RawJson` values and typed decoding.
- `src/trigger.rs` uses the shared `Timestamp` for context timestamps.
- `src/trigger.rs` defines `TriggerDelivery`, `TriggerContext`, and the
  SHA-256 body-hash identity fallback.
- `src/error.rs` defines bounded payload errors and their status mapping.

### Related Docs

- [Webhook protocol](../../docs/protocol/webhooks.md)
- [Connect resource protocol](../../docs/protocol/connect-resources.md)
- [SDK architecture](../../docs/architecture.md)
