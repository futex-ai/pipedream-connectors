# Token and usage operations

These operations create and validate Connect tokens, list usage, and obtain
OAuth client-credentials bearers.

## Operation inventory

| Method | Path | OpenAPI operationId |
| --- | --- | --- |
| POST | `/v1/connect/{project_id}/tokens` | `createToken` |
| GET | `/v1/connect/tokens/{ctok}/validate` | `validateToken` |
| GET | `/v1/connect/usage` | `listUsages` |
| POST | `/v1/oauth/token` | `createOauthToken` |

## Connect tokens

A Connect token belongs to one project, environment, and external user.

`expires_in` MUST be between 1 and 14,400 seconds inclusive.

The default Connect token lifetime is 14,400 seconds.

The default Connect token scope is `connect:*`.

A token request MAY include success and error redirect URIs, allowed origins,
a webhook URI, and progressive-scope policy.

Redirect and webhook URIs MUST use HTTPS.

The response contains a Connect Link URL, an expiration time, and the token.

The client MUST wrap the token and Connect Link URL as credential-class
values.

The token creation rate limit is 100 requests per minute per external user.

Development mode permits at most 10 unique external users.

## Account credential selection

Account list and retrieve operations accept `include_credentials`.

The default is false.

When true, Pipedream MAY return credentials for bring-your-own-OAuth apps.

The client MUST keep those credentials redacted in debug output.

## OAuth token operation

The OAuth request uses the client-credentials grant.

The OAuth response reports a one-hour bearer lifetime.

The client follows the refresh and invalidation rules in the [client
protocol](../connect-client.md).

Usage list responses follow the common cursor contract.
