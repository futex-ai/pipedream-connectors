# Connect client protocol

This document defines the server-side HTTP contract for Pipedream Connect.
The client sends Connect API requests to the Pipedream API origin and keeps
credentials on the server.

## Origin and transport

The default API origin is `https://api.pipedream.com`.

The configured API origin MUST use HTTPS.

The client MUST reject an origin with a username, password, fragment, or
control character.

The client MUST send Connect API requests to the configured origin and the
documented `/v1` paths. Workflow invocation is the exception: it sends the
bearer to the caller-named workflow endpoint.

The client MUST never follow an HTTP redirect.

The default connection timeout is 10 seconds.

The default total request timeout is 60 seconds.

The default response body limit is 10 MiB.

Callers MAY configure the connection timeout, total timeout, and response
limit.

**Decision.** The default origin is Pipedream's public HTTPS origin, and a
caller may select another HTTPS origin for a compatible service. A plain HTTP
default or an automatic downgrade was rejected because credentials and
response data must remain encrypted.

## OAuth client credentials

The client obtains a Pipedream API bearer by sending a JSON request to
`POST /v1/oauth/token`.

The request MUST contain `grant_type: "client_credentials"`, `client_id`, and
`client_secret`.

The request MAY contain a space-separated `scope`.

The response contains `access_token`, `token_type`, and `expires_in`.

Pipedream issues a bearer with a one-hour lifetime.

The client MUST refresh a cached bearer before expiry by the smaller of
30 seconds or 10 percent of the reported lifetime.

The client MUST allow only one refresh operation at a time.

Callers queued behind a failed refresh MUST receive that refresh failure.

A later caller MAY start another refresh without a hold-off interval.

Token expiry MUST use monotonic time and MUST ignore wall-clock changes.

A request that receives 401 MUST invalidate the bearer used for that request.

The client MUST return the 401 error without replaying the rejected request.

A static bearer provider MAY be used instead of client credentials.

A static bearer provider MUST never call the token endpoint.

**Decision.** The client caches and single-flights client-credentials tokens,
and a static bearer remains available for hosts that own token rotation.
Refreshing for every request was rejected because the token endpoint is rate
limited, and replaying the rejected request was rejected because a mutation
may already have run.

## Request headers

Every authenticated request MUST send `Authorization: Bearer <token>`.

Every Connect request MUST send `x-pd-environment` with `development` or
`production`.

The client MUST send `Content-Type: application/json` when a request has a
JSON body.

The client MUST send `Accept: application/json` for JSON operations.

The client MUST send a user-agent that identifies this SDK and its version.

The client MUST send `x-pd-rate-limit` when the caller configured a custom
rate-limit token.

The client MUST NOT send `x-pd-rate-limit` when no custom token is configured.

The OAuth token request MUST send the content type and user-agent headers but
MUST NOT require an environment header.

**Decision.** The environment is explicit on every environment-scoped call.
Inferring it from the project or process was rejected because development and
production credentials are separate.

## Success and error mapping

A 2xx response is successful when its body matches the operation contract.

A non-2xx JSON response uses the `{ "error", "code", "details" }` envelope
when Pipedream supplies that envelope.

The `error` value MAY contain text or structured JSON.

Every 202 response in execution mode is `Accepted`, including blocking
configure and props requests.

A 401 response is `Unauthorized`.

A 403 response is `Forbidden`.

A 404 response is `NotFound`.

A 429 response is `RateLimited`.

A `RateLimited` error includes parsed `Retry-After`,
`X-PD-Rate-Limit-Reset-In`, `X-RateLimit-Limit`, `X-RateLimit-Remaining`, and
`X-RateLimit-Reset` values when present.

An execution response containing an `error` object with `attribution` or
`name` is `ExecutionFailed` with the structured attribution and HTTP status.
Other structured error objects use the normal HTTP status mapping.

Proxy pass-through responses MUST preserve every status after checking only
non-2xx execution envelopes that contain `attribution`.

Proxy pass-through requests MUST never retry.

**Decision.** A proxy 401 invalidates its bearer and remains a proxy result.
Keeping the bearer was rejected because an upstream 401 and a Connect 401
cannot be distinguished reliably.

A 408 response is `Unavailable` with its parsed `Retry-After` delay when
present.

A 5xx response is `Unavailable` with its parsed `Retry-After` delay when
present.

Any other 3xx or 4xx response is `Rejected` with its status, error code, and
message when the response supplies them, unless an operation defines a
redirect result, as the file stash download does.

A timeout or transport failure is `Unavailable`.

A body larger than the configured response limit is `ResponseTooLarge` with the
response status and configured limit.

A local validation failure is `InvalidRequest` and sends no request.

An encoding, decoding, or client-construction failure is `Internal`.

A JSON decode failure on a success body MAY expose only the line and column of
the failure.

The client MUST NOT include a success body, request body, bearer, client
secret, signing key, or other credential-class value in that decode error.

**Decision.** The client keeps the documented public error categories and
parses the documented error envelope. Returning raw success-body text on a
decode failure was rejected because a success body can contain a token.

## Retry and rate-limit headers

The client MUST parse `Retry-After` as an integer number of seconds or as an
HTTP date.

The client MUST parse `X-RateLimit-Limit` as the request limit when present.

The client MUST parse `X-RateLimit-Remaining` as the remaining request count
when present.

The client MUST parse `X-RateLimit-Reset` as a Unix timestamp when present.

The client MUST parse `X-PD-Rate-Limit-Reset-In` as a reset delay in seconds
when present.

The client MUST expose absent or malformed rate-limit fields as absent values.

The client MUST NOT retry POST, PUT, PATCH, or DELETE automatically.

The client MAY enable retries for GET requests through an explicit policy.

The default GET retry policy has three total attempts, a 30-second maximum
delay, and a one-second fallback delay.

The default GET retry policy MUST remain opt-in.

The GET retry policy MUST select only 408, 429, and 5xx responses.

The GET retry policy MUST cap the total attempts.

The GET retry policy MUST honor a valid `Retry-After` value.

The GET retry policy MUST cap every delay at its configured maximum.

When a valid `Retry-After` exceeds that maximum, the client MUST stop retries
and return the mapped error with the parsed delay.

The GET retry policy MUST use an injected sleeper.

**Decision.** Mutating requests have no automatic replay because Connect does
not provide an idempotency key for these operations. GET retries are opt-in so
the host chooses its failure budget. A service-wide retry loop was rejected
because it could replay actions or proxy mutations.

## Credential redaction

The client MUST redact these credential-class values from debug output and
error text:

- OAuth client secrets.
- OAuth access tokens.
- Static bearer values.
- Connect tokens.
- Connect Link URLs that contain a Connect token.
- Webhook signing keys.
- Custom rate-limit tokens.
- Account `credentials` values.

The client MUST keep app-defined properties, proxy bodies, response bodies,
and provider error text available as normal data.

The client MUST require an explicit operation to expose a redacted value.

The client MUST redact credential-class values in nested structures.

**Decision.** Only credential-class values are wrapped. Redacting every app
field was rejected because callers need to inspect provider data. Wrapping
nothing was rejected because diagnostics can expose live credentials.

## Request and response bounds

The client MUST stop reading a response after the configured response limit.

The client MUST reject a response that announces a size above the limit.

The client MUST reject a response that crosses the limit while streaming.

The client MUST apply the total timeout to token, Connect, and proxy calls.

The client MUST apply the connection timeout before a connection is ready.

Callers MAY raise or lower each bound through client configuration.
