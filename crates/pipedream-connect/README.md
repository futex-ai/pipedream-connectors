# pipedream-connect

`pipedream-connect` provides asynchronous authentication and typed resources
for Pipedream Connect. Depend on this crate when a server needs authenticated
HTTP access. Use `pipedream-connect-types` for request and response models.

## Responsibilities

- Authenticate with OAuth client credentials or a host-managed bearer.
- Share token refresh across concurrent requests.
- Build API routes and apply the environment and authentication headers.
- Bound response bodies and map statuses to typed errors.
- Support explicit GET retry and pagination budgets.
- Discover apps, components, actions, and registry triggers.
- Deploy and manage external-user triggers, events, workflows, and webhooks.
- Configure properties, reload dynamic properties, and run actions once.
- Manage projects, custom OAuth clients, app overrides, and project webhooks.
- List Connect usage and create custom rate-limit tokens.
- Download File Stash files without following redirects.
- Invoke workflow endpoints for external users.
- Create and validate Connect tokens.
- List, create, retrieve, and delete connected accounts and external users.
- Send authenticated proxy requests with bounded, lossless byte responses.

## What This Crate Does

The default `reqwest` feature provides a rustls transport. It follows no
redirects and performs no transport retries. Its default connection timeout
is 10 seconds. Its default total request timeout is 60 seconds. Its default
response limit is 10 MiB. The builder can change each limit.

Resource accessors return shared trait objects. Each group has a `unimock`
mock under the `unimock` feature. Paged lists return `Page<T>` with provider
metadata. App, component, and deployed-trigger retrieval preserve their
response envelopes and check returned identities where the API provides the
requested identity. `app_categories()` lists the bare category array and
provides checked category retrieval. Deployed trigger calls keep the project
and external user scope. Delete can treat a missing deployment as success when
its missing-resource choice is enabled. Signing keys stay wrapped in
`SecretString`.

Configure, reload, and run calls use the shared execution error mapping.
Every 202 in execution mode maps to `Error::Accepted`, including blocking
configure and reload calls, without decoding a result body. These mutations
never retry.

`tokens()`, `accounts()`, and `users()` return managed-auth handles. Account
credential fields use `SecretJson`, so their `Debug` output is redacted.
Account list, retrieval, and user-account requests default
`include_credentials` to false.
Ownership helpers check the returned account partition before deletion.

`proxy()` returns a lossless pass-through handle. `ProxyRequest` encodes an
exact HTTPS target or relative path as URL-safe Base64 without padding. It
rejects reserved headers before authentication, prefixes allowed headers with
`x-pd-proxy-`, and sends body bytes unchanged. `ProxyResponse` preserves the
upstream status, every response header, and every body byte.

`RateLimited` retains the standard rate-limit fields, the `Retry-After` delay,
and the custom reset delay when Pipedream sends them. `Unavailable` retains
its response status and parsed `Retry-After` delay. `ResponseTooLarge` retains
the response status and configured byte limit.

`PipedreamClient::new(config, transport, clock, sleeper)` accepts shared trait
objects. It has no implicit Tokio sleeper. The builder supplies a Tokio
sleeper and a system clock with a monotonic origin.
The `HttpTransport`, `Clock`, `Sleeper`, and `AccessTokenProvider` traits each
have a mock under the `unimock` feature. Disable default features to use a
host transport without reqwest. Inject a custom sleeper or token provider
with the client methods before obtaining a dispatcher.

Apply a token returned by `rate_limits().create(...)` with
`client.with_rate_limit_token(token.token)`. The method returns a configured
client clone and keeps the token redacted in diagnostics.

OAuth refresh uses the smaller of 30 seconds or ten percent of the token
lifetime. A 401 invalidates the cached bearer without replaying the request.
A static bearer has no refresh cache. Its rotation belongs to the host.
Replace the provider or client when a static bearer changes.

Diagnostics redact credentials, HTTP bodies, headers, and query values.
Provider error messages remain available in typed error fields. Error
formatting does not print those messages. Success JSON decode errors retain
only the line and column.

GET retries are disabled until a caller supplies `RetryPolicy`. The default
policy has three total attempts, a 30-second maximum delay, and a one-second
fallback delay when `Retry-After` is absent. A valid `Retry-After` value above
the maximum stops retries and returns the mapped error with the parsed delay.
Fallback delays stay within the maximum. No method other than GET retries.
Transport failures do not trigger a retry.

`BoundedPaginator` uses an injected `PageSource` through `Paginator`. The
`FilteredPageSource` adapter fixes list filters and changes only the page. Its
`paginate` method returns records, page count, a stop reason, and a resumption
request when the page limit stops traversal. A page failure returns
`PaginationError::Interrupted` with the prior records, completed page count,
failed request, and original error. The host can resume from that request.
A repeated cursor stops traversal without sending another request.

## Quick Start

Construct the default client without sending a request:

```rust,no_run
#[cfg(feature = "reqwest")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use pipedream_connect::PipedreamClient;
    use pipedream_connect::config::ClientCredentials;
    use pipedream_connect_types::{Environment, ProjectId};

    let client = PipedreamClient::builder()
        .project_id(ProjectId::new("proj_Example123")?)
        .environment(Environment::Development)
        .client_credentials(ClientCredentials::new("client-id", "client-secret")?)
        .build()?;
    let _ = client;
    Ok(())
}
#[cfg(not(feature = "reqwest"))]
fn main() {}
```

Read an app and list its actions through the resource handles:

```rust,no_run
# async fn read_apps(client: pipedream_connect::PipedreamClient)
# -> Result<(), Box<dyn std::error::Error>> {
use pipedream_connect_types::{AppSlug, ListRegistryRequest, PageRequest};

let slug = AppSlug::new("google_calendar")?;
let app = client.apps().get(slug.clone()).await?;
let page = client.actions().list(ListRegistryRequest {
    app: Some(slug),
    page: PageRequest::with_limit(25)?,
    ..ListRegistryRequest::default()
}).await?;
# let _ = (app, page);
# Ok(())
# }
```

Deploy and manage a trigger through the lifecycle handle:

```rust,no_run
# async fn deploy_trigger(
#     client: pipedream_connect::PipedreamClient,
#     request: pipedream_connect_types::DeployTriggerRequest,
# ) -> Result<(), Box<dyn std::error::Error>> {
let external_user_id = request.external_user_id.clone();
let deployed = client.triggers().deploy(request).await?;
let _page = client.deployed_triggers().list(
    pipedream_connect_types::ListDeployedTriggersRequest {
        external_user_id,
        page: pipedream_connect_types::PageRequest::default(),
        emitter_type: None,
    },
).await?;
let _ = deployed;
# Ok(())
# }
```

For a custom operation, use `ApiPath::project` for routes under
`/v1/connect/{project_id}`. Pass each
path parameter as a separate segment. Query pairs preserve repeated keys.
For a JSON body, call `with_json(&model)`. The model must implement `Validate`.
This checks the model before the token call. `with_bytes` sends an exact byte
body with caller-supplied content headers.

`ResponseMode::Execution` enables execution errors and accepted 202 results.
`ResponseMode::Redirect` returns a redirect response without
following its location. Select these modes only for operations whose
contracts define them.

`ResponseMode::PassThrough` checks only non-2xx execution envelopes with
`attribution`, then returns every status, header, and body byte. It never
retries. A 401 still invalidates the bearer. The client cannot distinguish an
upstream 401 from a Connect 401. Workflow calls use a raw pass-through mode
that performs no execution check.

The URL library skips literal `.` and `..` segments. Their identifier policy
is still an open plan decision. Slash, percent, question mark, and hash
characters are encoded inside each segment.

Workflow invocations use the raw pass-through response mode and send the
external user and environment headers to the HTTPS endpoint. File Stash
downloads return bytes or a redirect location without following the redirect.

## Development

Run the targeted tests and the workspace gate:

```sh
cargo test -p pipedream-connect --all-features
cargo test -p pipedream-connect --no-default-features
cargo xtask check
```

The local-socket transport tests start a local HTTP server. They need no
Pipedream credentials. Unit tests use mocked transport, time, and sleep.
No unit test uses a real network or clock.

The `openapi_coverage` integration test reads the vendored OpenAPI fixture and
the protocol operation tables. It checks that the tables contain each covered
operation exactly once. It then calls the client method of each covered
OpenAPI operation through a recording transport and compares the method, full
URL, path template, key headers, and exact body with a golden wire row. The
rate-limit tokens and workflow invocation operations have no OpenAPI
operation, so this test does not cover them. The OAuth token row uses the
client-credentials provider so the test also covers the token request.

Run the registry and trigger smoke tests with the explicit opt-in:

```sh
PIPEDREAM_LIVE_TESTS=1 cargo test -p pipedream-connect \
  --test live_pipedream -- --nocapture
```

The registry smoke reads Google Calendar and lists its actions and triggers.
The trigger lifecycle smoke deploys, lists, and deletes one trigger. It always
cleans up a trigger created by the test. It requires
`PIPEDREAM_EXTERNAL_USER_ID` and `PIPEDREAM_CONNECTED_ACCOUNT_ID`. Both tests
print missing variable names and skip when credentials or the opt-in are
absent.

### Adding an operation

Add the typed request and response models first. Add a resource trait method
and a handle method. Put shared dispatch, no-content, and identity helpers in
`src/resources/operation.rs`. Put each area's pure route builders in its
per-area operation module. Use `ApiRequest::with_query` for query models
and `with_json` for JSON bodies. Decode through the shared operation helper.
For a paged list, wrap the fixed filters in `FilteredPageSource` and pass it
to `BoundedPaginator`:

```rust,no_run
use std::sync::Arc;
use pipedream_connect::pagination::{BoundedPaginator, FilteredPageSource, Paginator};
use pipedream_connect_types::{Component, ListRegistryRequest, PageRequest};

# async fn pages(
#     client: pipedream_connect::PipedreamClient,
# ) -> Result<(), Box<dyn std::error::Error>> {
let filters = ListRegistryRequest::default();
let source = FilteredPageSource::new(filters, move |request| {
    let actions = client.actions();
    async move { actions.list(request).await }
});
let paginator = BoundedPaginator::new(Arc::new(source));
let _pages = paginator.paginate(PageRequest::default(), std::num::NonZeroUsize::new(4).unwrap()).await?;
# let _: Option<Component> = None;
# Ok(())
# }
```

### Key Code

- `src/client.rs` defines `PipedreamClient` and `RequestDispatcher`.
- `src/config.rs` defines validated configuration and HTTP bounds.
- `src/builder.rs` constructs the default transport and clock.
- `src/request.rs` defines operations and the shared URL builder.
- `src/validation.rs` requires each JSON request model to own its validation.
- `src/resources/` defines the resource traits, handles, and client accessors.
- `src/resources/admin_operations.rs` builds administration and workflow
  requests and checks returned identities.
- `src/resources/projects.rs` manages projects and enabled project apps.
- `src/resources/oauth_apps.rs` and `src/resources/app_overrides.rs` manage
  project-owned OAuth clients and app overrides.
- `src/resources/project_webhooks.rs` manages environment webhooks and keys.
- `src/resources/file_stash.rs` preserves File Stash bytes and redirects.
- `src/resources/workflows.rs` invokes external-user workflow endpoints.
- `src/resources/operation.rs` builds, dispatches, decodes, and checks
  resource operations.
- `src/resources/trigger_operations.rs` builds trigger lifecycle requests and
  checks returned trigger and webhook identities.
- `src/resources/triggers.rs` deploys registry triggers.
- `src/resources/deployed_triggers.rs` manages deployed triggers, events,
  workflow endpoints, webhook destinations, and signing keys.
- `src/resources/auth_operations.rs` builds managed-auth and proxy requests,
  validates proxy targets and headers, and checks account identities.
- `src/resources/tokens.rs`, `accounts.rs`, and `users.rs` define managed-auth
  traits and handles.
- `src/resources/proxy.rs` defines proxy requests, methods, and responses.
- `src/token.rs` defines static and single-flight OAuth token providers.
- `src/transport.rs` defines the redacted HTTP boundary and safe JSON access.
- `src/reqwest_transport.rs` streams bounded HTTP responses.
- `src/runtime.rs` defines clock and sleeper boundaries.
- `src/retry.rs` defines explicit retry budgets without defaults.
- `src/pagination.rs` defines bounded page traversal and its stop reasons.
- `src/error.rs` defines the public error contract.
- `tests/http_transport.rs` exercises reqwest through local sockets.
- `tests/live_pipedream.rs` provides the credential-gated registry smoke test.

### Related Docs

- [Connect client protocol](../../docs/protocol/connect-client.md)
- [Connect resource protocol](../../docs/protocol/connect-resources.md)
- [Trigger resource protocol](../../docs/protocol/resources/triggers.md)
- [SDK architecture](../../docs/architecture.md)
