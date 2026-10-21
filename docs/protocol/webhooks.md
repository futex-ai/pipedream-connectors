# Connect webhook protocol

This document defines signature verification and webhook payloads for Connect
connection events and deployed trigger deliveries.

## Signature header

A signed request MUST contain one `x-pd-signature` header.

The header MUST have exactly two comma-separated parts in this form:

```text
t=<seconds>,v1=<hex>
```

`t` MUST contain a non-negative Unix timestamp in seconds.

`v1` MUST contain one 32-byte HMAC-SHA256 digest encoded as hexadecimal.

The signed message is the ASCII timestamp, a period, and the raw request body:

```text
<t>.<raw body>
```

The verifier MUST compute HMAC-SHA256 with the selected signing key.

The verifier MUST compare the supplied and computed digests in constant time.

The verifier MUST reject a missing `t` part.

The verifier MUST reject a missing `v1` part.

The verifier MUST reject duplicate `t` parts.

The verifier MUST reject duplicate `v1` parts.

The verifier MUST reject unknown parts, empty values, malformed timestamps, and
malformed hexadecimal values.

The verifier MUST reject an empty signing key.

The verifier MUST use the request receipt time for replay checking.

The default allowed timestamp difference is 300 seconds in either direction.

The verifier MUST reject a timestamp older than 300 seconds from receipt.

The verifier MUST reject a timestamp more than 300 seconds in the future.

The caller MAY configure a different absolute tolerance.

The verifier MUST read the raw body before JSON parsing.

The raw webhook body limit is configurable.

The verifier MUST reject a body larger than the configured limit.

The verifier MUST NOT include the signature header, signing key, or raw body in
a signature error.

**Decision.** The verifier authenticates the exact bytes Pipedream sent and uses
constant-time comparison with a 300-second two-sided window. Parsing decoded
JSON or accepting one-sided age checks was rejected because either change can
permit body changes or replayed future requests.

## Signing-key selection

A deployed trigger webhook has a trigger-level signing key when Pipedream
returns one for that destination.

The receiving host MUST select the trigger-level key for a delivery sent to
that destination.

The receiving host MUST select the project environment webhook signing key
when the delivery has no trigger-level key.

The project environment key MUST match the `x-pd-environment` value used for
the destination.

The verifier MUST verify the signature with the signing key supplied by the
receiving host.

A missing key MUST fail verification.

A key MUST remain a credential-class value in diagnostics and storage.

**Decision.** Trigger-level keys take precedence over the project environment
key. Using only a project-wide key was rejected because a trigger can have a
different destination and signing secret.

## Connection events

A connection webhook is a JSON POST with an `event` field.

`event` MUST be `CONNECTION_SUCCESS` or `CONNECTION_ERROR`.

A success payload contains `connect_token`, `environment`,
`connect_session_id`, and an `account` object.

The success `account` object contains the connected account identity and app
metadata.

A success payload MUST NOT contain user credentials.

A consumer MAY retrieve credentials through the account operation when its
policy permits `include_credentials`.

An error payload contains `connect_token`, `environment`,
`connect_session_id`, and an `error` message.

The webhook decoder MUST wrap the Connect token in either payload as a
credential-class value.

The client MUST preserve unknown fields in either connection payload.

The connection event session ID MAY be a large numeric value and MUST remain
lossless during decoding.

## Trigger deliveries

A trigger delivery is a JSON object with an `event` field and a `context` field.

The `event` field contains the app-defined event JSON without lossy conversion.

The `context` field MAY contain an `id` string.

The `context` field MAY contain a timestamp in an ISO 8601 string.

The consumer MUST preserve the raw event JSON and unknown context fields.

A delivery with a context ID uses that ID as its event identity.

A delivery without a context ID uses the SHA-256 digest of the raw body as its
event identity.

An invalid context ID MUST reject the delivery.

The body cap applies before payload processing.

**Decision.** Trigger event data stays app-defined JSON, and a missing context
ID falls back to a body digest. Requiring a provider-specific event schema was
rejected because every integration can emit a different event shape.
