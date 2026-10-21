# pipedream-connectors

Status: Typed models, webhooks, the client core, registry and execution
resources, managed auth and proxy resources, trigger lifecycle resources,
and administration resources are available. Category listing returns a
bare category array. The `pipedream-cli` command line runs registry,
execution, managed auth, proxy, trigger, and administration commands.

This repository will provide a Rust SDK for the Pipedream Connect API. The
SDK will manage server-side OAuth client credentials, typed Connect resources,
cursor pages, authenticated proxy calls, and webhook verification.

The SDK uses three library crates:

- `crates/pipedream-connect-types` for pure models and lossless data.
- `crates/pipedream-connect` for the asynchronous client and resources.
- `crates/pipedream-connect-webhooks` for signature verification and payloads.

The unpublished `crates/pipedream-cli` crate provides the `pipedream-cli`
command line.

## Quick start

The client can authenticate and use typed resources. Construct it with an
explicit project and environment:

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

The [client crate README](crates/pipedream-connect/README.md) shows the shared
resource handles and its injectable runtime boundaries. The registry handles
list apps, components, actions, and triggers. Component, action, and trigger
handles also configure and reload properties. The trigger handle deploys a
trigger. The deployed-trigger handle lists, updates, and deletes deployments,
reads events, manages workflows and webhooks, and regenerates signing keys.
The action handle runs configured actions. Managed-auth handles create and
validate Connect tokens, manage accounts and users, and support authenticated
proxy calls.

The client uses `https://api.pipedream.com` by default. It also manages
projects, OAuth clients, app overrides, project webhooks, usage, rate-limit
tokens, File Stash downloads, and workflow invocation. Set
`PIPEDREAM_CLIENT_ID`, `PIPEDREAM_CLIENT_SECRET`, `PIPEDREAM_PROJECT_ID`, and
`PIPEDREAM_LIVE_TESTS=1` for registry smoke commands. Set
`PIPEDREAM_EXTERNAL_USER_ID` and `PIPEDREAM_CONNECTED_ACCOUNT_ID` for the
trigger lifecycle smoke. The live checks skip when a required variable is
absent.

## CLI

The `pipedream-cli` command runs SDK operations and prints each result as
JSON. It reads settings from flags or from the `PIPEDREAM_CLIENT_ID`,
`PIPEDREAM_CLIENT_SECRET`, `PIPEDREAM_PROJECT_ID`, `PIPEDREAM_ENVIRONMENT`,
and `PIPEDREAM_API_ORIGIN` variables. It redacts credentials unless the
caller passes `--show-secrets`. Each failure has a documented exit code.

```sh
export PIPEDREAM_ENVIRONMENT=development
cargo run -p pipedream-cli -- apps list --query calendar
cargo run -p pipedream-cli -- components list --app google_calendar --type action
cargo run -p pipedream-cli -- actions run google_calendar-list-events \
  --external-user-id user_1 --props props.json
cargo run -p pipedream-cli -- tokens create --external-user-id user_1
```

The read commands are `apps list`, `apps get`, `categories list`,
`components list`, `components get`, `actions list`, `accounts list`,
`accounts get`, `users list`, `proxy get`, `triggers list`, `triggers get`,
`triggers events`, `project info`, `usage`, and `webhook get`. The commands
that change data are `actions run`, `tokens create`, `proxy post`, `put`,
`patch`, and `delete`, `triggers deploy`, and `triggers delete`. The CLI sends
each change once. The [CLI README](crates/pipedream-cli/README.md) is the
command reference.

## Developer start

The workspace and `xtask` gate are available. The library crates are under
active development. To work on the repository:

1. Install the pinned Rust toolchain.
2. Copy the required Pipedream variables into the shell environment.
3. Run `cargo xtask check`.
4. Run the credential-gated live checks when a development project exists.
5. Run `cargo test -p pipedream-cli --test cli_smoke` for the CLI smoke
   without credentials.
6. Run `npx --yes markdownlint-cli2` on changed Markdown files.

Do not commit credentials. Keep scratch output under `.context/`.

## Key code

The main entry points are:

- `crates/pipedream-connect-types/src/ids.rs` for validated identifiers.
- `crates/pipedream-connect-types/src/secret.rs` for redacted credential
  wrappers.
- `crates/pipedream-connect-types/src/pagination.rs` for bounded cursor pages.
- `crates/pipedream-connect-types/src/registry/` for app and component models.
- `crates/pipedream-connect-types/src/props/` for lossless property models.
- `crates/pipedream-connect-types/tests/conformance.rs` for schema-generated
  OpenAPI conformance samples.
- `crates/pipedream-connect` for client configuration, token providers,
  transport boundaries, resource operations, proxy calls, and errors.
- `crates/pipedream-connect/src/resources/` for registry and execution traits,
  handles, query encoding, and the client accessors.
- `crates/pipedream-connect-webhooks` for raw-body signature checks and
  webhook payload decoding.
- `crates/pipedream-cli` for the command line, its JSON output, and its exit
  codes.
- `xtask` for the complete repository gate.

## Documentation

Read the [architecture](docs/architecture.md) before changing a component
boundary.

Read the [client protocol](docs/protocol/connect-client.md) for authentication,
headers, errors, limits, retries, and redaction.

Read the [resources protocol](docs/protocol/connect-resources.md) for operation
coverage, pagination, lossless models, and proxy behaviour.

Read the [webhook protocol](docs/protocol/webhooks.md) for signatures and
payloads.

Read the [plans directory](plans/) for milestone status and decisions.
