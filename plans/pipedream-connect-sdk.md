# Pipedream Connect Rust SDK

Status: Active. The plan lives on branch
`calummoore/pipedream-rust-sdk-plan`. No pull request is open yet. The plan
closes when its PR merges.

Pipedream Connect is a hosted integration API. It gives a server access to
more than 3,000 third-party apps: managed OAuth accounts, prebuilt actions,
deployable triggers, and an authenticated API proxy. This repository builds
a standalone Rust SDK for that API. The SDK gives typed models for the whole
Connect REST API. It gives an async client with OAuth client-credentials
authentication and an injectable HTTP transport. It bounds responses, types
errors, pages with cursors, supports the proxy, and verifies webhooks.

The SDK generalizes an existing adapter. Firna, the private product in
`futex-ai/firna`, has the crate `fna-connectors-pipedream` on branch
`calummoore/tel-aviv-v4`. That crate is bound to Firna's provider-neutral
connector interface and to Firna product policy. It cannot be reused outside
Firna. The SDK ports the proven mechanics of that adapter. These are
single-flight token refresh, redacted diagnostics, bounded streaming bodies,
strict identifier grammars, proxy header admission, and signature
verification. The SDK drops Firna policy: the provider-neutral trait, product
redaction of app data, service-side retries, and the closed prop model that
omits components. Firna adopts the SDK in a later, separate change in its own
repository.

Research evidence lives under `.context/pipedream-connect-sdk/`:
`research-notes.md`, `openapi-operations.md`, and the vendored
`pipedream_openapi_swagger.json` fetched on 2026-10-20.

Contract owners, created in Milestone 1:

- `docs/protocol/connect-client.md`: authentication, environment, request
  and response contract, errors, limits, retries, and redaction.
- `docs/protocol/connect-resources.md`: the operations the SDK covers and
  their typed contracts, pagination, and the proxy.
- `docs/protocol/webhooks.md`: signature verification, connection events,
  and trigger deliveries.
- `docs/architecture.md`: the crates and their boundaries.
- The workspace `README.md` and each crate `README.md`.

## Non-goals

- The Pipedream MCP server transport.
- The legacy Pipedream REST API for the account's own workflows, sources,
  subscriptions, and events. The SDK covers only `/v1/connect/*`,
  `/v1/oauth/token`, the custom rate-limit token, and workflow invocation for
  external users.
- The browser-side Connect flow. The SDK is a server-side client. It creates
  Connect tokens and Connect Link URLs for a frontend to use.
- Product redaction of app-defined data, retries of non-idempotent
  operations, persistence, and catalogue sync. These stay in the host.
- The Firna adoption itself. It is a post-merge follow-up in `futex-ai/firna`.

## Open decisions (proposed 2026-10-20, awaiting the user)

The milestones below apply the recommended option of each decision until
the user answers. A different answer updates this plan and the protocol
docs before the affected milestone starts.

### 1. Crate layout and names

- **Severity:** Medium. **Effort:** Large.
- **Background:** The Firna adapter is split into an interface crate with
  types and a client crate. The SDK has three kinds of code: pure data
  models, the HTTP client, and webhook verification. Webhook receivers and
  persistence layers need the models but not the client. The crates.io
  names `pipedream-connect`, `pipedream-connect-types`, and
  `pipedream-connect-webhooks` are free. `pipedream` and `pipedream-rs` are
  taken by unrelated crates.
- **Problem:** The crate split decides the public API shape, the number of
  READMEs, and what a consumer must depend on.
- **Impact if no action:** Implementation cannot start.
- **Options:**
  - **A.** One crate `pipedream-connect` with cargo features `reqwest`,
    `webhooks`, and `unimock`. Simplest dependency. Every consumer compiles
    the client code. Effort: Large.
  - **B.** Two crates: `pipedream-connect` with models and client, and
    `pipedream-connect-webhooks`. The webhooks crate depends on the client
    crate without its HTTP feature, so it still pulls in tokio. Effort: Large.
  - **C.** Three crates: `pipedream-connect-types` with the pure models,
    `pipedream-connect` with the client, and `pipedream-connect-webhooks`.
    Consumers of the client add two dependencies. Effort: Large.
- **Recommendation:** C. The types crate has no async runtime, and webhook
  receivers and stores use it without the client.

### 2. Error runtime and publishing target

- **Severity:** Medium. **Effort:** Medium.
- **Terms:**
  - **internal-error:** The shared futex crate that gives `InternalError`
    and the `ErrorContract` derive. It lives at `futex-ai/internal-error`
    and is not on crates.io.
- **Background:** The shared Rust rules require a `thiserror` error enum
  with an `Internal(InternalError)` fallback from `internal-error`. Other
  futex libraries, for example `futex-ai/ai` and `futex-ai/redloop`, use it
  as a git dependency and are themselves consumed by git revision. Firna
  pins `internal-error` at revision `6477922`. A crate on crates.io cannot
  have git dependencies.
- **Problem:** The SDK must choose between the shared error contract and a
  crates.io release.
- **Impact if no action:** The error design and the manifests are blocked.
- **Options:**
  - **A.** Depend on `internal-error` by git revision `6477922`, the same as
    Firna. Consume the SDK by git revision. No crates.io release in V1.
    Effort: Medium.
  - **B.** Define a small self-contained internal error type in the types
    crate. No futex dependencies. Publish to crates.io. The error contract
    differs from the other futex crates. Effort: Medium.
- **Recommendation:** A. It keeps one error contract across futex crates and
  matches how Firna consumes shared libraries. Publishing becomes a
  follow-up when `internal-error` reaches crates.io.

### 3. API surface in V1

- **Severity:** Medium. **Effort:** Large.
- **Background:** The Pipedream OpenAPI document lists 72 operations. The
  `/v1/connect/*` group holds 68 of them. The Firna adapter uses 21. Two
  documented features are not in the OpenAPI document: the custom rate-limit
  token and workflow invocation for external users. The file
  `.context/pipedream-connect-sdk/openapi-operations.md` lists every
  operation.
- **Problem:** Each resource group costs typed models, a resource trait,
  tests, and docs.
- **Impact if no action:** The milestones cannot be sized.
- **Options:**
  - **A.** Firna parity only: registry, components, configure, props, run,
    proxy, tokens, accounts, users, deployed triggers, webhooks, events, and
    usage. Effort: Large.
  - **B.** Every `/v1/connect/*` operation plus the OAuth token, the
    rate-limit token, and workflow invocation. Effort: Large.
  - **C.** B plus the legacy `/v1/apps` and `/v1/app_categories` routes,
    which duplicate the Connect routes. Effort: Large.
- **Recommendation:** B. An SDK that covers the documented API needs no
  second pass, and the extra groups are small CRUD resources.

### 4. Transport defaults and retries

- **Severity:** Medium. **Effort:** Medium.
- **Background:** The Firna adapter never follows redirects and never
  retries. It uses a 10-second connect timeout and a 30-second total
  timeout. It caps every response at 1 MiB. Firna retries idempotent reads
  in its service layer. The official Pipedream SDKs retry 408, 429, and 5xx
  responses twice with backoff and use a 60-second timeout. The Pipedream
  proxy allows upstream calls of up to 30 seconds.
- **Problem:** A general SDK needs safe defaults that callers can change.
- **Impact if no action:** Callers either lose data to a fixed cap or
  replay non-idempotent requests.
- **Options:**
  - **A.** Keep the Firna values fixed and not configurable. Effort: Small.
  - **B.** Configurable client builder with safe defaults: no redirects, no
    retries, 10-second connect timeout, 60-second total timeout, 10 MiB
    response cap. An opt-in retry policy applies only to GET requests,
    honours `Retry-After`, caps the delay, and uses an injected sleeper.
    Effort: Medium.
  - **C.** Official SDK behaviour: two retries on 408, 429, and 5xx for every
    method, 60-second timeout, no response cap. Effort: Medium.
- **Recommendation:** B. It never replays a mutation, and callers tune the
  limits for their workload.

### 5. Secret wrapping granularity

- **Severity:** Low. **Effort:** Medium.
- **Terms:**
  - **SecretString:** A string wrapper whose `Debug` output is `[REDACTED]`
    and whose value needs an explicit `expose_secret` call.
- **Background:** The Firna adapter wraps credentials and also app-defined
  data in `SecretString`: option labels, proxy bodies, event payloads, and
  provider error text. Firna then applies product redaction. An SDK consumer
  works with the data directly.
- **Problem:** Broad wrapping makes every field awkward to use. No wrapping
  leaks tokens into logs.
- **Impact if no action:** Either an unusable API or credential leaks in
  `Debug` output.
- **Options:**
  - **A.** Wrap only credential-class values: client secret, access tokens,
    Connect tokens, Connect Link URLs, signing keys, rate-limit tokens, and
    account `credentials`. Keep app data and provider error text as plain
    types. Effort: Medium.
  - **B.** Wrap everything the Firna adapter wraps. Effort: Medium.
  - **C.** Wrap nothing. Effort: Small.
- **Recommendation:** A. Credentials never reach logs, and consumers read
  app data without ceremony.

### 6. Smoke tooling

- **Severity:** Low. **Effort:** Small.
- **Background:** The repository rules need a smoke test for every feature.
  The Firna adapter has a credential-gated live test that reads the Google
  Calendar registry in a development project.
- **Problem:** The SDK needs a way to run against the real API.
- **Impact if no action:** Features ship without a real-API check.
- **Options:**
  - **A.** Cargo examples under `crates/pipedream-connect/examples/` plus
    credential-gated live tests that skip without credentials. Effort: Small.
  - **B.** A `pipedream-cli` binary crate with `clap` subcommands for the
    read-only operations. Effort: Large.
- **Recommendation:** A. Examples double as documentation and need no new
  crate.

## Design decisions (2026-10-20)

These decisions follow the shared rules or the Pipedream contract. They need
no user answer. Each records the alternative and the reason.

- **Lossless models.** Unknown prop types become `PropKind::Other` with the
  raw definition. Unknown enum values, for example an auth type or an
  emitter type, become an `Other(String)` variant. Rejected: the Firna
  closed model that omits whole components. Reason: an SDK must not drop
  data that the API returns.
- **One identifier grammar.** Each identifier type validates the same
  grammar on construction and on deserialization: visible ASCII, at most 250
  bytes, no URL separators. Documented prefixes are enforced: `proj_`,
  `apn_`, `dc_`, `oa_`, `appcat_`, and `ctok_` with 32 lowercase hex digits.
  Rejected: free strings. Reason: identifiers go into URL paths.
- **Resource traits.** One object-safe async trait per resource group, for
  example `AccountsApi` and `DeployedTriggersApi`. The client returns a
  handle per group that implements its trait. Each trait gets a `unimock`
  mock behind the `unimock` cargo feature. Rejected: one trait with every
  method. Reason: smaller mocks and smaller files.
- **Injected transport and clock.** `HttpTransport` and `Clock` are traits.
  The reqwest transport is behind the default `reqwest` feature. Rejected:
  a hard reqwest dependency. Reason: unit tests and other HTTP stacks.
- **Token handling.** The client-credentials provider caches the bearer. It
  refreshes early by the smaller of 30 seconds or 10 percent of the
  lifetime. It serializes refresh behind one mutex. It invalidates a bearer
  that gets a 401. The rejected request is never replayed. A static bearer
  provider also exists. Rejected: refresh per request. Reason: the token
  endpoint is rate limited.
- **No automatic replay.** The client never retries a POST, PUT, PATCH, or
  DELETE. Reason: actions and proxy calls have no idempotency key.
- **Proxy requests are lossless.** The SDK encodes the target as URL-safe
  base64 without padding. It rejects Pipedream's restricted headers and any
  caller `x-pd-*` header before the token call. It prefixes allowed headers
  with `x-pd-proxy-` and sends bytes. It returns the upstream status, every
  header, and the body bytes. Rejected: the Firna safe-header filter.
  Reason: the host decides what to publish.
- **Error envelope.** Non-2xx responses decode the documented
  `{ error, code, details }` envelope. The structured execution envelope
  `{ error: { attribution, name, message } }` becomes `ExecutionFailed`.
  A JSON decode failure of a success body becomes an internal error that
  keeps only the line and column. Reason: a body may hold a token.
- **Dates.** ISO 8601 strings and epoch milliseconds decode to
  `chrono::DateTime<Utc>`. Event timestamps keep the raw number and offer a
  conversion.
- **Toolchain.** Edition 2024, MIT licence, `rust-toolchain.toml` pinned to
  the same channel as `futex-ai/ai`, `cargo xtask check` as the gate.

## Public API sketch

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
    .send(ProxyRequest::get(account, "https://slack.com/api/conversations.list"))
    .await?;
```

`PipedreamClient::new(config, transport, clock)` takes
`Arc<dyn HttpTransport>` and `Arc<dyn Clock>` for composition roots and
tests. The builder constructs the reqwest transport and the system clock.

The public error contract of the client crate:

- `InvalidRequest(InvalidRequest)`: local validation failed on an
  identifier, a URL, the header policy, or a limit. No request was sent.
- `Unauthorized`, `Forbidden`, `NotFound`: a 401, 403, or 404 with the
  envelope message.
- `RateLimited { retry_after, limit, remaining, reset }`: a 429 with the
  parsed headers.
- `Rejected { status, code, message }`: any other 4xx.
- `ExecutionFailed(ExecutionError)`: structured attribution from an action
  or proxy run.
- `Accepted`: a 202 from configure, props, or run when `blocking` is false.
- `Unavailable { status }`: a 408, a 5xx, a timeout, or a transport
  failure. The request may have run.
- `ResponseTooLarge { limit }`: the body passed the configured cap.
- `Internal(InternalError)`: an encode, decode, or client construction
  failure.

## Milestone 1: Documentation and protocol

Summary: write the requirements and the decisions before any code, with the
recommended options applied.

- [ ] Write `docs/protocol/connect-client.md`:
  - [ ] OAuth client-credentials token acquisition, the one-hour lifetime,
        and early refresh.
  - [ ] The `x-pd-environment` header, the optional `x-pd-rate-limit`
        header, and the other request headers.
  - [ ] Status mapping to the error contract, `Retry-After` and
        `X-RateLimit-*` parsing.
  - [ ] Response caps, timeouts, the no-replay rule, and the opt-in GET
        retry policy.
  - [ ] The redaction rule for credential-class values.
- [ ] Write `docs/protocol/connect-resources.md`:
  - [ ] The covered operations grouped by resource.
  - [ ] Cursor pagination with `limit`, `after`, and `before`.
  - [ ] Lossless models and identifier grammars.
  - [ ] Connect token limits and `include_credentials`.
  - [ ] Proxy encoding and header policy, and the 202 contract.
  - [ ] The trigger lifecycle operations.
- [ ] Write `docs/protocol/webhooks.md`:
  - [ ] `x-pd-signature` verification, the 300-second default tolerance,
        and the body cap.
  - [ ] Connection event payloads and trigger delivery payloads.
  - [ ] Key selection between trigger webhooks and the project environment
        webhook.
- [ ] Write `docs/architecture.md`: the three crates, the transport and
      clock boundaries, the token provider, and the host responsibilities.
- [ ] Write the workspace `README.md`: features, quick start, developer
      get-started, key code, and links to the docs and `plans/`.
- [ ] Validate the Markdown, review the diff, commit, and push.

## Milestone 2: Repository bootstrap

Summary: a compiling workspace with the gate and CI, no SDK code yet.

- [ ] Create the Cargo workspace: `crates/pipedream-connect-types`,
      `crates/pipedream-connect`, `crates/pipedream-connect-webhooks`, and
      `xtask`, each with a `README.md` in the required section order.
- [ ] Add `rust-toolchain.toml`, `.cargo/config.toml` with the `xtask`
      alias, `LICENSE`, and `.gitignore` entries for `target` and `.context/`.
- [ ] Add `internal-error` as a workspace git dependency at revision
      `6477922`, and the shared external dependencies with `cargo add`.
- [ ] Implement `cargo xtask check` with these steps:
  - [ ] `cargo fmt --all -- --check`.
  - [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
  - [ ] `cargo test --workspace --all-features`.
  - [ ] `cargo doc --no-deps` with warnings denied.
  - [ ] A Rust file-length lint with the 300-line cap.
- [ ] Add `.github/workflows/ci.yml` that runs the same steps on pull
      requests and on `main`.
- [ ] Create `AGENTS.md` from the shared template: commands, repository
      rules, review focus, and status. Add `CLAUDE.md` as a symlink to it.
- [ ] Run the gate, commit, and push.

## Milestone 3: Typed models

Summary: the `pipedream-connect-types` crate, pure data with serde.

- [ ] Identifiers with one grammar for construction and deserialization:
      `ProjectId`, `ExternalUserId`, `AccountId`, `DeployedTriggerId`,
      `OauthAppId`, `AppCategoryId`, `AppSlug`, `ComponentKey`, `PropName`,
      `WebhookId`, `ConnectToken`, and `Cursor`.
- [ ] `SecretString` and `SecretJson` with redacted `Debug` and explicit
      exposure, and `Environment`.
- [ ] Pagination: `PageRequest` with limit 1 to 200, `PageInfo`, `Page<T>`,
      and `PageRequest::next` that detects a stuck cursor.
- [ ] Registry models: `App`, `AppAuthType`, `ProxySupport`, `AppCategory`,
      `ListAppsRequest` with sort and filters, `Component`, `ComponentType`,
      `ToolAnnotations`, `ComponentStash`, and `ListComponentsRequest`.
- [ ] Prop models:
  - [ ] `ConfigurableProp` with the shared base fields and a lossless
        `PropKind`.
  - [ ] `PropOption` for labelled, nested, and scalar choices, and
        `TimerValue`.
  - [ ] `ConfiguredProps` with a typed `ConfiguredPropValue` that keeps
        app, SQL, and raw JSON values.
- [ ] Managed-auth models: `Account` with the nested `App` and optional
      credentials, `CreateAccountRequest`, `ListAccountsRequest`,
      `CreateConnectTokenRequest` with `expires_in` 1 to 14,400,
      `ConnectTokenResponse`, `ValidateTokenResponse`, and `ExternalUser`.
- [ ] Execution models: `ConfigurePropRequest` and response,
      `ReloadPropsRequest` and response with `DynamicProps`,
      `RunActionRequest` with `stash_id`, `RunActionResponse`,
      `Observation`, and `ExecutionError` with the attribution origin.
- [ ] Trigger models:
  - [ ] `DeployTriggerRequest` and the `Emitter` enum for deployed
        components, HTTP interfaces, and timer interfaces.
  - [ ] `UpdateTriggerRequest`, `TriggerWebhook`, `TriggerWebhooks`, and
        `TriggerWorkflows`.
  - [ ] `EmittedEvent` and `ListDeployedTriggersRequest` with
        `emitter_type`.
- [ ] Administration models: `Project`, `ProjectInfo`, `OauthApp`,
      `AppOverride`, `ProjectWebhook`, `UsageRecord`, `RateLimitRequest`,
      `RateLimitToken`, and the create and update requests.
- [ ] Wire envelopes: `Data<T>`, `PageEnvelope<T>`, and `ErrorResponse`.
- [ ] Vendor the OpenAPI document under the crate `tests/fixtures/` with a
      refresh script. Add a conformance test that decodes every response
      example in it into the matching model.
- [ ] Unit tests for every grammar, lossless decoding of unknown kinds,
      redacted `Debug`, and pagination. Write the crate README.
- [ ] Run the gate, commit, and push.

## Milestone 4: Client core

Summary: the `pipedream-connect` crate can authenticate and dispatch, with
no resource yet.

- [ ] `ClientConfig` and the builder:
  - [ ] HTTPS origin validation, project, and environment.
  - [ ] Credentials or a static bearer, scope, and the rate-limit token.
  - [ ] Timeouts, the response cap, and the retry policy.
- [ ] `Clock` trait with `SystemClock`, and `HttpTransport` with redacted
      `HttpRequest` and `HttpResponse` types.
- [ ] `ReqwestTransport` behind the `reqwest` feature: rustls, no redirects,
      no client retries, configured timeouts, and a streamed body that stops
      at the cap.
- [ ] `AccessTokenProvider` trait with the client-credentials provider and
      the static provider, including early refresh, single flight, and
      invalidation on 401.
- [ ] Shared dispatch:
  - [ ] URL building under `/v1/connect/{project_id}` and the headers.
  - [ ] Status mapping to the error contract, `Retry-After` and
        `X-RateLimit-*` parsing, and the execution envelope.
  - [ ] Bounded JSON decoding.
- [ ] Opt-in `RetryPolicy` for GET requests with an injected `Sleeper`,
      honouring `Retry-After` and a maximum delay.
- [ ] A bounded `paginate` helper that drives `PageRequest::next` up to a
      page limit.
- [ ] Unit tests with a scripted unimock transport and clock:
  - [ ] Concurrent refresh, early refresh, and 401 invalidation without
        replay.
  - [ ] Every status mapping and the rate-limit header parsing.
  - [ ] Cap enforcement and retry counts checked by recorded requests.
- [ ] A local-socket integration test for the reqwest transport: exact
      headers, announced and streamed oversize bodies, and a redirect that
      is not followed. Write the crate README.
- [ ] Run the gate, commit, and push.

## Milestone 5: Registry and execution resources

Summary: apps, components, actions, and registry triggers work against the
real API.

- [ ] `AppsApi`: list with every filter, get by slug with an identity check.
- [ ] `AppCategoriesApi`: list and get.
- [ ] `ComponentsApi`: list with `component_type`, get with `version`,
      configure a prop, and reload props.
- [ ] `ActionsApi`: list, get, configure, reload props, and run with
      `blocking`, `stash_id`, and the 202 contract.
- [ ] `TriggersApi` for registry triggers: list, get, configure, and reload
      props.
- [ ] Unit tests per resource: query encoding, request bodies, response
      mapping, lossless unknown kinds, and error mapping.
- [ ] A credential-gated live test that reads the `google_calendar` app and
      lists its actions and triggers in a development project. It skips
      without `PIPEDREAM_CLIENT_ID`, `PIPEDREAM_CLIENT_SECRET`, and
      `PIPEDREAM_PROJECT_ID`.
- [ ] The `list_apps` example. Update the READMEs and the resources protocol
      doc only if a contract changed.
- [ ] Run the gate, run the example, commit, and push.

## Milestone 6: Managed auth and proxy resources

Summary: tokens, accounts, users, and the proxy work against the real API.

- [ ] `TokensApi`: create with URL validation of the redirect and webhook
      URIs, and validate.
- [ ] `AccountsApi`: list with filters, get with and without credentials,
      create from a Connect token, and delete. Delete all accounts of an app
      for the project.
- [ ] `UsersApi`: list, delete, and list the accounts of one external user.
- [ ] `ProxyApi`: GET, POST, PUT, PATCH, and DELETE with the header policy,
      the base64 target, byte bodies, and the lossless response.
- [ ] `ProjectsApi::info` for the apps enabled in the project environment.
- [ ] Unit tests: ownership checks, `include_credentials` handling with
      redacted `Debug`, every blocked header, prefixing, target encoding,
      and upstream error statuses preserved.
- [ ] The `connect_link` example that creates a token and prints the Connect
      Link URL for an external user.
- [ ] Run the gate, run the example against a development project, commit,
      and push.

## Milestone 7: Trigger lifecycle resources

Summary: deploy and manage triggers for an external user.

- [ ] `TriggersApi::deploy` with `webhook_url`, `workflow_id`,
      `emit_on_deploy`, and the `Emitter` response.
- [ ] `DeployedTriggersApi`:
  - [ ] List with `emitter_type`, get, update, and delete with
        `ignore_hook_errors`.
  - [ ] List events with `n`, and get and set workflows.
  - [ ] List and set webhooks, get one webhook, and regenerate its signing
        key.
- [ ] Unit tests: tenant scoping on every call, idempotent delete of a
      missing trigger, epoch-millisecond timestamps, signing-key redaction,
      and the emitter variants.
- [ ] A credential-gated live test that deploys a timer-free Google
      Calendar trigger to a loopback webhook URL, lists it, and deletes it.
- [ ] Run the gate, commit, and push.

## Milestone 8: Administration resources

Summary: the remaining Connect operations.

- [ ] `ProjectsApi`: list, create, get, update, delete, and update logo.
- [ ] `OauthAppsApi` and `AppOverridesApi`: list, create, get, update, and
      delete.
- [ ] `ProjectWebhooksApi`: get, set, delete, and regenerate the signing
      key for the project environment webhook.
- [ ] `UsageApi::list` for a time window, and `RateLimitsApi::create` that
      returns the token for the `x-pd-rate-limit` header.
- [ ] `FileStashApi::download` that returns bytes or the redirect location
      without following it.
- [ ] `WorkflowsApi::invoke_for_external_user` against a workflow endpoint
      URL with the external user and environment headers.
- [ ] A coverage test that reads the vendored OpenAPI document and asserts
      that every `operationId` appears in the SDK coverage table.
- [ ] Unit tests per resource. Update the READMEs.
- [ ] Run the gate, commit, and push.

## Milestone 9: Webhook verification

Summary: the `pipedream-connect-webhooks` crate.

- [ ] `WebhookSignatureVerifier` trait with the HMAC-SHA256 verifier:
  - [ ] Exact `t=<seconds>,v1=<hex>` parsing and constant-time comparison.
  - [ ] A configurable tolerance with a 300-second default, a body cap, and
        an explicit receipt time.
- [ ] `ConnectionEvent` for `CONNECTION_SUCCESS` and `CONNECTION_ERROR`
      with the account, Connect token, environment, and session id.
- [ ] `TriggerDelivery` with the event payload and the context id and
      timestamp, and a fallback event id from the body hash.
- [ ] Unit tests with fixed timestamps: valid, stale, future, duplicate
      header parts, bad hex, empty key, and oversize body. Write the crate
      README.
- [ ] Run the gate, commit, and push.

## Milestone 10: Examples, smoke, and release readiness

Summary: the SDK is usable by a stranger.

- [ ] Examples: `list_apps`, `connect_link`, `run_action`, `proxy_request`,
      and `deploy_trigger`, each reading configuration from environment
      variables.
- [ ] Each crate `lib.rs` includes its README as crate docs, so the Quick
      Start compiles as a doctest.
- [ ] `cargo doc --no-deps` builds without warnings, and `Cargo.toml`
      carries description, repository, keywords, and categories.
- [ ] Run the complete smoke against a development project: every example
      and both live tests. Save the output under
      `.context/pipedream-connect-sdk/smoke.md`.
- [ ] Update the workspace README and the architecture doc for anything
      learnt.
- [ ] Run the gate, commit, and push.

## Milestone 11: Verification, commit, and review

Summary: the final checks and the review loop.

- [ ] Run the targeted tests and the complete gate named in `AGENTS.md`.
- [ ] Commit with Conventional Commits and push the branch.
- [ ] Review the complete local diff against `origin/main` with the
      `implementation-review` skill after the push. Report the findings,
      then apply the review-fix rule: fix the `Auto-fix: yes` findings,
      re-review once, and report the rest.
  - Open findings go here by severity, High first, one line each: number,
    title, severity, effort, and recommended option. Follow-ups for wider
    fixes that the review did not apply, and problems outside the scope, go
    here too, one line each.

## Post-merge follow-up (non-blocking)

- [ ] Open a plan in `futex-ai/firna` that makes `fna-connectors-pipedream`
      depend on the SDK and keeps its `ConnectorProvider` mapping.
- [ ] Publish the crates to crates.io when `internal-error` is published,
      if decision 2 stays at option A.
- [ ] Consider an MCP server client and the legacy workflow REST API as
      separate plans.
