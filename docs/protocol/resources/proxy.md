# Proxy and workflow operations

These operations send authenticated requests through Connect and invoke a
workflow for an external user.

## Operation inventory

| Method | Path | OpenAPI operationId |
| --- | --- | --- |
| GET | `/v1/connect/{project_id}/proxy/{url_64}` | `retrieveProxy` |
| POST | `/v1/connect/{project_id}/proxy/{url_64}` | `postProxy` |
| PUT | `/v1/connect/{project_id}/proxy/{url_64}` | `putProxy` |
| PATCH | `/v1/connect/{project_id}/proxy/{url_64}` | `patchProxy` |
| DELETE | `/v1/connect/{project_id}/proxy/{url_64}` | `deleteProxy` |
| POST | `/v1/connect/rate_limits` | _Documented, no OpenAPI operationId_ |
| Any | Workflow endpoint URL | _Documented, no OpenAPI operationId_ |

## Proxy target and request

The proxy target is an absolute HTTPS URL or a relative path accepted by the
selected app.

The client MUST encode the exact target as URL-safe Base64 without padding.

The encoded target MUST occupy the `{url_64}` path segment.

The request MUST include the external user ID and MAY include an account ID.

The request body is an uninterpreted byte sequence.

The proxy MUST limit upstream execution to 30 seconds and returns 504 when the
upstream exceeds that limit.

The client MUST preserve the upstream status, every response header, and the
response body bytes.

The client MUST return upstream non-2xx statuses as proxy results when the
Connect API returns the response successfully.

## Proxy headers

The caller MAY supply headers for the upstream request.

The client MUST reject a caller header whose name starts with `x-pd-`.

The client MUST reject these restricted header names, case-insensitively:

- `Accept-Encoding`.
- `Access-Control-Request-Headers`.
- `Access-Control-Request-Method`.
- `Connection`.
- `Content-Length`.
- `Cookie`.
- `Date`.
- `DNT`.
- `Expect`.
- `Host`.
- `Keep-Alive`.
- `Origin`.
- `Permissions-Policy`.
- `Referer`.
- `TE`.
- `Trailer`.
- `Transfer-Encoding`.
- `Upgrade`.
- `Via`.
- Any name starting with `Proxy-`.
- Any name starting with `Sec-`.

The client MUST send every other caller header to Connect with the
`x-pd-proxy-` prefix.

The client MUST reject a restricted header before it requests a bearer.

**Decision.** The host controls the allowed upstream data, and the client
preserves the upstream response losslessly. The Firna safe-header allow-list
was rejected because it silently drops provider-specific headers. Caller
`x-pd-*` names are rejected because they are reserved for Connect metadata.

## Rate-limit token

`POST /v1/connect/rate_limits` returns a custom rate-limit token.

The request body uses `window_size_seconds` for the window duration in seconds.

The request body uses `requests_per_window` for the maximum request count.

Pipedream documents these fields in its [custom rate-limit
contract](https://pipedream.com/docs/connect/api-reference/rate-limits).

The client MUST preserve that token as a credential-class value.

The client MAY send the token in `x-pd-rate-limit` on later Connect requests.

The rate-limit token operation is documented by Pipedream but is absent from
the OpenAPI document.

## Workflow invocation

Workflow invocation sends the request to the workflow endpoint URL.

The request MAY use GET, POST, PUT, DELETE, or PATCH.

The request MUST send `Authorization: Bearer <access token>`.

The request MUST send `X-PD-External-User-ID` with the external user ID.

The request MUST send `X-PD-Environment` with `development` or `production`.

The request body is an uninterpreted byte sequence.

The workflow operation is documented by Pipedream but is absent from the
OpenAPI document.
