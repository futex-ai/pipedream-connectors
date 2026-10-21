<!-- markdownlint-disable MD041 -->

Status: The crates are under construction under the Pipedream Connect SDK
plan.

# pipedream-connectors

This repository will provide a Rust SDK for the Pipedream Connect API. The
SDK will manage server-side OAuth client credentials, typed Connect resources,
cursor pages, authenticated proxy calls, and webhook verification.

The SDK will use three library crates:

- `crates/pipedream-connect-types` for pure models and lossless data.
- `crates/pipedream-connect` for the asynchronous client and resources.
- `crates/pipedream-connect-webhooks` for signature verification and payloads.

Milestone 10 will add the unpublished `crates/pipedream-cli` command.

## Quick start

The public API is planned. The following sketch shows the intended client
flow after the crates are bootstrapped:

```rust
let client = PipedreamClient::builder()
    .project_id(ProjectId::new("proj_abc123")?)
    .environment(Environment::Development)
    .client_credentials(ClientCredentials::new(client_id, client_secret)?)
    .build()?;

let page = client.apps().list(ListAppsRequest::default()).await?;
let next = page.page_info.next(&PageRequest::default())?;

let token = client
    .tokens()
    .create(CreateConnectTokenRequest::new(external_user_id))
    .await?;

let response = client
    .proxy()
    .send(ProxyRequest::get(
        account,
        "https://slack.com/api/conversations.list",
    ))
    .await?;
```

The client will use `https://api.pipedream.com` by default. Set
`PIPEDREAM_CLIENT_ID`, `PIPEDREAM_CLIENT_SECRET`, and
`PIPEDREAM_PROJECT_ID` for development smoke commands. The live checks skip
when these variables are absent.

## Planned CLI

The planned CLI will read credentials from flags or `PIPEDREAM_*` variables
and print JSON.

```text
pipedream-cli apps list --query calendar
pipedream-cli components list --app google_calendar --type action
pipedream-cli tokens create --external-user-id user_1
pipedream-cli proxy get --external-user-id user_1 --account apn_x URL
pipedream-cli triggers deploy --external-user-id user_1 \
  --component google_calendar-new-or-updated-event-instant \
  --props props.json --webhook-url https://example.com/hook
```

The CLI will also cover component configuration, action runs, proxy requests,
trigger updates and deletes, account operations, project administration, and
webhook operations.

## Developer start

Milestone 2 will create the compiling Cargo workspace and the `xtask` gate.
After that milestone:

1. Install the pinned Rust toolchain.
2. Copy the required Pipedream variables into the shell environment.
3. Run `cargo xtask check`.
4. Run the credential-gated live checks when a development project exists.
5. Run `npx --yes markdownlint-cli2` on changed Markdown files.

Do not commit credentials. Keep scratch output under `.context/`.

## Key code

The planned entry points are:

- `crates/pipedream-connect-types` for identifiers, models, pagination,
  credential wrappers, and public data contracts.
- `crates/pipedream-connect` for client configuration, token providers,
  transport boundaries, resource operations, proxy calls, and errors.
- `crates/pipedream-connect-webhooks` for raw-body signature checks and
  webhook payload decoding.
- `crates/pipedream-cli` for smoke commands and JSON output.
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
