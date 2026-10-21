# pipedream-connect

`pipedream-connect` provides asynchronous authentication, registry discovery,
property configuration, and action execution for Pipedream Connect. Depend
on this crate when a server needs authenticated HTTP access. Use
`pipedream-connect-types` for request and response models.

## Responsibilities

- Authenticate with OAuth client credentials or a host-managed bearer.
- Share token refresh across concurrent requests.
- Build API routes and apply the environment and authentication headers.
- Bound response bodies and map statuses to typed errors.
- Support explicit GET retry and pagination budgets.
- Discover apps, components, actions, and registry triggers.
- Configure properties, reload dynamic properties, and run actions once.

## What This Crate Does

The default `reqwest` feature provides a rustls transport. It follows no
redirects and performs no transport retries. Its default connection timeout
is 10 seconds. Its default total request timeout is 60 seconds. Its default
response limit is 10 MiB. The builder can change each limit.

`apps()`, `components()`, `actions()`, and `triggers()` return shared resource
trait objects. Each group has a `unimock` mock under the `unimock` feature.
Paged lists return `Page<T>` with provider metadata. App and component
retrieval returns the complete `Data<T>` envelope and checks its identity.
`app_categories()` lists the bare category array and provides checked category
retrieval.

Configure, reload, and run calls use the shared execution error mapping.
An omitted or explicit `blocking: Some(false)` maps a 202 from configure or
reload to `Error::Accepted` without decoding a result body. Action runs never
send a blocking flag, so every 202 from a run is `Error::Accepted`. These
mutations never retry.

`PipedreamClient::new(config, transport, clock, sleeper)` accepts shared trait
objects. It has no implicit Tokio sleeper. The builder supplies a Tokio
sleeper and a system clock with a monotonic origin.
The `HttpTransport`, `Clock`, `Sleeper`, and `AccessTokenProvider` traits each
have a mock under the `unimock` feature. Disable default features to use a
host transport without reqwest. Inject a custom sleeper or token provider
with the client methods before obtaining a dispatcher.

OAuth refresh uses the smaller of 30 seconds or ten percent of the token
lifetime. A 401 invalidates the cached bearer without replaying the request.
A static bearer has no refresh cache. Its rotation belongs to the host.
Replace the provider or client when a static bearer changes.

Diagnostics redact credentials, HTTP bodies, headers, and query values.
Provider error messages remain available in typed error fields. Error
formatting does not print those messages. Success JSON decode errors retain
only the line and column.

GET retries are disabled until a caller supplies `RetryPolicy`. The policy
requires total attempts, a maximum delay, and a fallback delay. It honors a
valid `Retry-After` value. A value above the maximum stops retries and returns
the mapped error. Fallback delays stay within the maximum. No method
other than GET retries. Transport failures do not trigger a retry.

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
use pipedream_connect::PipedreamClient;
use pipedream_connect::config::ClientCredentials;
use pipedream_connect_types::{Environment, ProjectId};

let client = PipedreamClient::builder()
    .project_id(ProjectId::new("proj_Example123")?)
    .environment(Environment::Development)
    .client_credentials(ClientCredentials::new("client-id", "client-secret")?)
    .build()?;
# Ok::<(), Box<dyn std::error::Error>>(())
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

For a custom operation, use `ApiPath::project` for routes under
`/v1/connect/{project_id}`. Pass each
path parameter as a separate segment. Query pairs preserve repeated keys.
For a JSON body, call `with_json(&model)`. The model must implement `Validate`.
This checks the model before the token call. `with_bytes` sends an exact byte
body with caller-supplied content headers.

`ResponseMode::Execution` enables execution errors and nonblocking 202
results. `ResponseMode::Redirect` returns a redirect response without
following its location. Select these modes only for operations whose
contracts define them.

`ResponseMode::PassThrough` checks structured execution errors, then returns
every status, header, and body byte. It never retries. A 401 still invalidates
the bearer. The client cannot distinguish an upstream 401 from a Connect 401.

The URL library skips literal `.` and `..` segments. Their identifier policy
is still an open plan decision. Slash, percent, question mark, and hash
characters are encoded inside each segment.

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

Run the read-only registry smoke test with all three credentials and the
explicit opt-in:

```sh
PIPEDREAM_LIVE_TESTS=1 cargo test -p pipedream-connect \
  --test live_pipedream -- --nocapture
```

It reads Google Calendar and lists its actions and triggers in development.
It prints the missing variable names and skips when credentials or the opt-in
are absent.

### Adding an operation

Add the typed request and response models first. Add a resource trait method
and a handle method. Put the pure request builder and identity check in
`src/resources/operation.rs`. Use `ApiRequest::with_query` for query models
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
- `src/resources/operation.rs` builds, dispatches, decodes, and checks
  resource operations.
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
- [SDK architecture](../../docs/architecture.md)
