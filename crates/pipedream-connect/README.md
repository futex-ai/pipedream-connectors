# pipedream-connect

`pipedream-connect` provides asynchronous authentication and request dispatch
for Pipedream Connect. Depend on this crate when a server needs authenticated
HTTP access. Use `pipedream-connect-types` for request and response models.
Resource handles will arrive in the next SDK milestones.

## Responsibilities

- Authenticate with OAuth client credentials or a host-managed bearer.
- Share token refresh across concurrent requests.
- Build API routes and apply the environment and authentication headers.
- Bound response bodies and map statuses to typed errors.
- Support explicit GET retry and pagination budgets.

## What This Crate Does

The default `reqwest` feature provides a rustls transport. It follows no
redirects and performs no transport retries. Its default connection timeout
is 10 seconds. Its default total request timeout is 60 seconds. Its default
response limit is 10 MiB. The builder can change each limit.

`PipedreamClient::new(config, transport, clock)` accepts shared trait objects.
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
valid `Retry-After` value. Every delay stays within the maximum. No method
other than GET retries. Transport failures do not trigger a retry.

`BoundedPaginator` uses an injected `PageSource` through `Paginator`. Its
`paginate` method returns records, page count, a stop reason, and a resumption
request when the page limit stops traversal. A repeated cursor stops
traversal without sending another request.

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

Use the shared dispatcher until resource handles are available:

```rust,no_run
# async fn read_apps(client: pipedream_connect::PipedreamClient)
# -> Result<(), Box<dyn std::error::Error>> {
use http::Method;
use pipedream_connect::request::{ApiPath, ApiRequest};
use pipedream_connect_types::{App, Page, PageRequest};

let request = ApiRequest::new(Method::GET, ApiPath::connect(&["apps"]))
    .with_page(&PageRequest::with_limit(25)?)?;
let response = client.dispatcher().dispatch(request).await?;
let page: Page<App> = response.json()?;
# let _ = page;
# Ok(())
# }
```

Use `ApiPath::project` for routes under `/v1/connect/{project_id}`. Pass each
path parameter as a separate segment. Query pairs preserve repeated keys.
For a JSON body, call `with_json(&model, model.validate())`. This checks the
model before the token call. `with_bytes` sends an exact byte body with
caller-supplied content headers.

`ResponseMode::Execution` enables execution errors and nonblocking 202
results. `ResponseMode::Redirect` returns a redirect response without
following its location. Select these modes only for operations whose
contracts define them.

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

### Key Code

- `src/client.rs` defines `PipedreamClient` and `RequestDispatcher`.
- `src/config.rs` defines validated configuration and HTTP bounds.
- `src/builder.rs` constructs the default transport and clock.
- `src/request.rs` defines operations and the shared URL builder.
- `src/token.rs` defines static and single-flight OAuth token providers.
- `src/transport.rs` defines the redacted HTTP boundary and safe JSON access.
- `src/reqwest_transport.rs` streams bounded HTTP responses.
- `src/runtime.rs` defines clock and sleeper boundaries.
- `src/retry.rs` defines explicit retry budgets without defaults.
- `src/pagination.rs` defines bounded page traversal and its stop reasons.
- `src/error.rs` defines the public error contract.
- `tests/http_transport.rs` exercises reqwest through local sockets.

### Related Docs

- [Connect client protocol](../../docs/protocol/connect-client.md)
- [Connect resource protocol](../../docs/protocol/connect-resources.md)
- [SDK architecture](../../docs/architecture.md)
