# Pipedream Connect SDK architecture

The SDK is a server-side Rust workspace for Pipedream Connect. It separates
pure data from HTTP access and webhook ingress so a host can choose only the
boundaries it needs.

## Components

The types library owns pure identifiers, request and response models, lossless
JSON values, pagination metadata, credential wrappers, and public error data.
It has no network, clock, sleeper, or async-runtime responsibility.

The client library owns authenticated Connect requests, resource operations,
response bounds, status mapping, token providers, and optional GET retry
policy. It depends on the types library and exposes transport boundaries rather
than requiring one HTTP implementation.

The webhooks library owns signature verification, replay-window checks, raw
body limits, connection event decoding, and trigger delivery decoding. It
shares the pure models and credential wrappers with the client boundary.
It keeps session identifiers, event payloads, and unknown webhook fields as
exact JSON text. It offers typed access without changing JSON number handling
in applications that depend on the SDK.

The CLI is a user-facing composition root. It reads configuration from flags
or `PIPEDREAM_*` environment variables, constructs a client, runs read
commands and explicit smoke mutations, and prints JSON results. It is not a
published library.

The `xtask` task runner owns the repository gate. It runs formatting, linting,
tests, documentation checks, and the file-length check as one developer
command.

## Runtime boundaries

The HTTP transport boundary accepts a method, URL, headers, and byte body. It
returns a status, headers, and bounded byte body. The client does not depend on
an HTTP library at this boundary.

The default transport uses an HTTPS client with redirects disabled, the
configured connection and total timeouts, and a streaming response limit.

The clock boundary supplies monotonic time for token expiry and wall time for
HTTP-date retry delays. The webhook verifier receives an explicit receipt
time from its host, so it does not read a clock.

The sleeper boundary waits between opt-in GET retry attempts. Tests provide a
fake sleeper, and the host can provide a runtime sleeper.

The transport, clock, and sleeper are shared trait objects at the composition
root. Resource operations receive these boundaries through the client.

## Authentication flow

The host supplies either client credentials or a static bearer.

The client-credentials provider requests a one-hour OAuth bearer, caches its
expiry, refreshes early, and serializes concurrent refreshes.

The request dispatcher asks the provider for a bearer before an authenticated
request.

A 401 invalidates the used bearer and returns the error without replaying the
request.

The dispatcher adds the environment, content type, user-agent, and optional
custom rate-limit header before it calls the transport.

## Data flow

A host builds a client with a project, environment, origin, credentials, and
transport boundaries.

A resource operation validates identifiers, URLs, limits, and proxy headers.

The client obtains a bearer, builds the documented Connect path, and sends the
request through the transport.

The response reader enforces the byte cap before status mapping and success
decoding.

The dispatcher maps HTTP statuses and structured execution errors before
success decoding. Shared JSON decoding retains only safe error locations.
The resource operation returns a typed lossless model, a byte-preserving proxy
response, an accepted asynchronous result, or a public client error.

A webhook host receives the raw HTTP body and signature header. It selects the
trigger or project environment key, verifies the raw bytes, and then decodes a
connection or trigger payload.

## Host responsibilities

The host stores client credentials, static bearers, Connect tokens, signing
keys, and account credentials in its own secret store.

The host chooses a transport, clock, sleeper, retry policy, and persistence
strategy at the composition root.

The host decides which app-defined fields to log, store, redact, or publish.

The host owns HTTP server admission, request body buffering, response status
codes, replay deduplication, authorization of callers, and webhook delivery
acknowledgement.

The host owns application retries for mutations, job queues, and durable
pagination checkpoints.

The host owns the browser-side Connect Link flow. The SDK creates the token and
link but does not expose credentials to browser code.

## Documentation boundaries

The [client protocol](protocol/connect-client.md) defines wire headers,
errors, limits, authentication, and retries.

The [resources protocol](protocol/connect-resources.md) defines operation
coverage, models, pagination, and proxy semantics.

The [webhook protocol](protocol/webhooks.md) defines signatures and ingress
payloads.
