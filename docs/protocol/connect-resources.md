# Connect resources protocol

This document defines the resource coverage and data contracts for the SDK.
It covers Pipedream Connect routes, the OAuth token route, the documented
rate-limit token route, and workflow invocation for external users.

The operation inventory is split into these pages:

- [Catalog and execution](resources/catalog.md).
- [Accounts and users](resources/accounts.md).
- [Projects and administration](resources/projects.md).
- [Triggers and deployed triggers](resources/triggers.md).
- [Proxy and workflows](resources/proxy.md).
- [Tokens and usage](resources/tokens.md).

The inventory contains every operation in the OpenAPI document that belongs to
`/v1/connect/*`, plus `POST /v1/oauth/token`.

The legacy `/v1/apps`, `/v1/apps/{app_id}`, `/v1/app_categories`, and
`/v1/app_categories/{id}` routes are outside this SDK.

## Lossless data

The SDK MUST preserve every JSON field that Pipedream returns.

The SDK MUST preserve unknown fields when it decodes an object.

The SDK MUST preserve an unknown property definition with its raw JSON value.

The SDK MUST preserve an unknown enum value as its original string.

The SDK MUST preserve unknown response fields when it re-encodes a model.

The SDK MUST NOT discard a component because its property type is unknown.

The SDK MUST preserve app-defined property values without product redaction.

Typed models MUST preserve integer values exactly only when they fit in the
`i64` or `u64` range. Larger integers and long decimal values can round during
decoding or re-encoding.

Webhook payload fields MUST preserve the exact JSON text supplied by Pipedream.

Date fields MUST accept RFC 3339 strings and epoch-millisecond numbers.

A date field MUST re-encode in the wire form that it decoded. When a field that
the OpenAPI document types as an integer decodes an integer value, it MUST
re-encode that value as an integer.

**Decision.** Unknown property types keep their raw definition, and unknown
enum values keep their strings. A closed model that drops new components or
values was rejected because a general SDK must remain lossless as Pipedream
adds data.

## Identifiers

An identifier MUST contain visible ASCII bytes only.

An identifier MUST be between 1 and 250 bytes inclusive.

An identifier MUST NOT contain a URL separator, backslash, query marker,
fragment marker, or control character.

An identifier MUST NOT be exactly `.` or `..`.

The identifier grammar MUST apply both during construction and during JSON
deserialization.

The following prefixes and patterns are part of the contract.

<!-- markdownlint-disable MD013 -->

| Identifier | Pattern | Evidence |
| --- | --- | --- |
| Project | `proj_[0-9A-Za-z]+` | OpenAPI pattern and Connect examples |
| Account | `apn_[a-zA-Z0-9]+` | OpenAPI pattern and Connect proxy examples |
| Deployed component | `dc_[a-zA-Z0-9]+` | OpenAPI pattern; official docs do not show a value |
| HTTP interface emitter | `hi_[a-zA-Z0-9]+` | OpenAPI response pattern |
| Timer interface emitter | `ti_[a-zA-Z0-9]+` | OpenAPI response pattern |
| OAuth app | `oa_[0-9A-Za-z]+` | OpenAPI pattern and webhook examples |
| App category | `appcat_[a-zA-Z0-9]+` | OpenAPI pattern; official docs do not show a value |
| Connect token | `ctok_[0-9a-f]{32}` | OpenAPI pattern and Connect Link examples |

<!-- markdownlint-enable MD013 -->

The SDK MUST enforce the documented prefix pattern for each identifier above.

The SDK MUST use the common grammar for external user IDs, app slugs, app
override IDs, component keys, property names, webhook IDs, and workflow
endpoint IDs unless this protocol specifies a stricter pattern.

The OpenAPI document shows workflow endpoint IDs with a `p_` prefix, but the
Connect docs describe endpoint URLs rather than a stable ID grammar.

**Decision.** The listed eight prefixes (`proj_`, `apn_`, `dc_`, `hi_`, `ti_`,
`oa_`, `appcat_`, and `ctok_`) are enforced because the OpenAPI document
specifies them. A deployed trigger ID takes `dc_`, `hi_`, or `ti_`. App override
IDs use the common grammar even though the OpenAPI document shows `^ao_`. The
SDK adds no prefix rule beyond the eight listed prefixes without a user
decision. `p_` is accepted as an opaque workflow endpoint value until
Pipedream publishes a stable workflow ID contract.
A free-form grammar for all IDs was rejected because these values enter URL
paths. Dot segments were rejected because URL path normalization could route a
request to another resource.

## Cursor pagination

A cursor MUST contain visible ASCII bytes only.

A cursor MUST be between 1 and 2048 bytes inclusive.

A cursor MAY contain any visible ASCII character because it is opaque and is
sent only as a query parameter.

A paged request MAY include `limit`, `after`, or `before` query parameters.

`limit` MUST be between 1 and 200 inclusive.

The default `limit` is 100.

`after` requests rows after the supplied opaque cursor.

`before` requests rows before the supplied opaque cursor.

A request MUST NOT include both `after` and `before`.

A paged response contains `data` and `page_info`.

`page_info` MAY contain `count`, `total_count`, `start_cursor`, and
`end_cursor`.

A missing `count` MUST NOT mean that the page is empty.

A forward page MAY use `end_cursor` as the next `after` value.

The client MUST reject a next page when its cursor equals the request's
`after` cursor.

The client MUST stop pagination when `count` is zero.

The client MUST stop pagination when the page has no next cursor.

The client MUST preserve the provider count and cursors even when a model is
unknown.

**Decision.** The page limit is 1 through 200 with a default of 100. A larger
unbounded page was rejected because response memory and latency must remain
bounded. A repeated cursor is an error because silently looping would consume
requests without making progress.

## Common operation rules

Every operation that uses a project ID MUST scope the path to that project.

Every operation that uses an external user ID MUST keep that value in the
request body or query specified by its operation contract.

The client MUST validate local identifiers, URLs, header names, and limits
before it requests a bearer.

A response body MUST be decoded without dropping unknown fields.

The [client protocol](connect-client.md) defines authentication, errors,
limits, retries, and redaction for every operation.
