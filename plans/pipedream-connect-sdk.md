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
`pipedream_openapi_swagger.json` fetched on 2026-10-20. A read-only sparse
clone of the Firna connector crates at commit `905976b` lives under
`.context/firna/`.

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

## Decisions (settled 2026-10-20)

The user answered every decision on 2026-10-20. Each item keeps its options
as the record of the alternatives. The **Outcome** line gives the choice.
The milestones apply the outcomes.

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
- **Outcome:** C, chosen by the user on 2026-10-20.

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
- **Outcome:** A, chosen by the user on 2026-10-20.

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
- **Outcome:** B, chosen by the user on 2026-10-20.

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
- **Outcome:** B, chosen by the user on 2026-10-20.

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
- **Outcome:** A, chosen by the user on 2026-10-20.

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
- **Outcome:** B, chosen by the user on 2026-10-20. The recommendation was
  A. The CLI scope is a design decision below.

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
- **CLI scope.** The `pipedream-cli` crate covers the read operations. It
  also covers the explicit smoke mutations: token creation, action run,
  proxy request, and trigger deploy and delete. Every mutation needs its
  own subcommand.
  The CLI reads configuration from flags or the `PIPEDREAM_*` environment
  variables and prints JSON. It depends on the resource traits, not on
  the concrete client. It is never published. Rejected: read-only commands
  plus separate examples. Reason: one tool serves every smoke test. Decided
  by the plan author on 2026-10-20 after decision 6.
- **CLI milestone (2026-10-21).** The CLI is the user interface of the SDK.
  All CLI work moves into Milestone 10, a `Tags: ui` milestone after the
  resource and webhook milestones. Milestones 5 to 8 keep only library
  work. Rejected: CLI commands inside each resource milestone. Reason: the
  plans rule keeps UI and backend work in separate milestones, and one
  milestone gives the CLI one consistent design. Decided by the implementer
  on 2026-10-21.

- **OpenAPI operation inventory (2026-10-21).** The fetched OpenAPI document
  contains 67 `/v1/connect/*` operations and one OAuth token operation. The
  resources protocol lists those 68 operation IDs once, plus the documented
  rate-limit token and workflow invocation operations without invented IDs.
  Rejected: adding the four legacy `/v1/apps` and `/v1/app_categories`
  operations or inventing IDs for undocumented operations. Reason: decision
  3 selects the Connect surface, and the OpenAPI document is the source for
  operation IDs.
- **Identifier prefix evidence (2026-10-21).** The OpenAPI patterns confirm
  `proj_`, `apn_`, `dc_`, `oa_`, `appcat_`, and `ctok_`. The current official
  Connect pages show examples for `proj_`, `apn_`, `oa_`, and `ctok_`, but do
  not show `dc_` or `appcat_` values. The protocol reports those two prefixes
  as OpenAPI-only evidence. Rejected: treating an absent documentation
  example as proof that the OpenAPI pattern is wrong. Reason: the API schema
  is the stronger contract for these identifiers.

- **Secret wire serialization (2026-10-21).** `SecretString`, `SecretJson`,
  Connect tokens, and credential fields serialize their inner values because
  request and response bodies need the exact wire value. Their `Debug`
  implementations stay redacted, and callers need `expose_secret` for code
  access. Rejected: omitting secret fields from serialization, because the
  client could not send or round-trip credential-class values.
- **Pagination next API (2026-10-21).** `PageRequest::next(&PageInfo)` is the
  one pagination advancement API. It supports both cursor directions and
  rejects a repeated cursor. Rejected: a method on `Page<T>`, because request
  state belongs to `PageRequest` and response models stay pure.
- **Schema-generated conformance samples (2026-10-21).** The conformance
  test generates one JSON instance from every covered response schema,
  including required and optional fields, types, formats, enums, and a small
  identifier-pattern table. Key-based samples also match existing identifier
  types when a field schema omits their prefix. It decodes each instance.
  It checks that documented fields do not enter unknown-field maps, and checks
  a lossless round trip.
  It also decodes future `example` and `examples` response values. Rejected:
  testing only response examples, because the fetched OpenAPI document has no
  response examples.

- **Webhook ingress inputs (2026-10-21).** The verifier requires a caller
  supplied body limit and receipt `SystemTime`, and the payload decoders offer
  bounded entry points. Webhook JSON uses arbitrary-precision numbers so large
  session IDs and app events remain lossless. Rejected: an ambient clock, an
  invented body-limit default, or floating-point JSON, because hosts own
  ingress admission and the protocol requires lossless payloads.

- **Review fix round, model ownership (2026-10-21).** Model modules use named
  files and explicit exports. One crate-private string-enum macro encodes
  known and unknown values as raw strings. `Page<T>` owns every paged
  response. Rejected: duplicated enum serializers, broad exports, and a
  second page envelope. Reason: one implementation prevents wire drift.
- **Review fix round, lossless values (2026-10-21).** Configured numbers use
  `serde_json::Number`. App, SQL, timer, and option variants retain extra
  object fields. SQL decoding uses named schema fields. Unknown prop kinds
  parse shared fields on a best-effort basis and retain failed fields.
  Rejected: floating-point integers and strict parsing of future prop
  definitions. Reason: both choices lose valid provider data.
- **Review fix round, optional fields (2026-10-21).** Absent response
  quantities, flags, lists, and maps use `Option` instead of fabricated defaults.
  Null prop fields remain in the extra map when no typed value exists.
  `ProjectInfoApp.name_slug` is optional, and option page numbers use
  `serde_json::Number` as their schemas specify. App override updates retain
  explicit null OAuth selections. Rejected: empty or false defaults and
  treating null updates as omitted updates. Reason: those values can change
  the meaning of a response or update.
- **Review fix round, request selectors (2026-10-21).** File Stash selectors
  distinguish IDs, `NEW`, and boolean values. Both boolean values remain
  accepted because the schema permits them. Query models carry the
  operation-specific filters, versions, and external user values. Rejected:
  string-only selectors and untyped query maps. Reason: typed requests must
  cover every documented wire shape without new rejections.
- **Review fix round, webhook JSON (2026-10-21).** Exact `RawJson` text with
  typed accessors replaces arbitrary-precision `serde_json` handling.
  Session IDs, app events, and unknown webhook fields keep their original
  JSON number text. This supersedes the arbitrary-precision choice above.
  Rejected: a workspace-wide number feature or floating-point event JSON.
  Reason: an SDK must preserve event data without changing host JSON
  handling through Cargo feature unification. Direct serde decoding also
  initializes the private trigger identity. Rejected: an empty identity
  outside the bounded decoder. Reason: every decoded delivery needs a stable
  identity.
- **Review fix round, conformance (2026-10-21).** Schema tests cover all
  union branches, populated maps, nulls, and omitted optional fields.
  Assertions inspect nested extra maps and compare encoded request bodies
  and queries. Known prop-kind details remain in their lossless extra maps.
  Key-based identifier samples cover `project_id`, `oauth_app_id`,
  `account_id`, `connect_token`, and app category IDs. Some field schemas omit
  prefixes that the model types enforce. The `appcat_` prefix decision remains
  open, and these samples retain the current model rule without settling it.
  Rejected: one sample per union and identifiers derived only from schema
  examples. Reason: all documented branches need checks, and prefix omissions
  must not prevent tests of the other fields. These samples do not prove that
  a model accepts every identifier its schema permits.
- **Dispatch composition (2026-10-21).** `RequestDispatcher` owns the shared
  authenticated request flow. `PipedreamClient` constructs it from shared
  transport, clock, sleeper, and token-provider traits. Pure operation
  descriptions build URLs in one place. The shared response JSON decoder
  retains only line and column on failure. Rejected: resource-specific HTTP
  stacks, because they could diverge on headers, limits, or authentication.
- **Explicit retry inputs (2026-10-21).** `RetryPolicy` requires total
  attempts, maximum delay, and a fallback delay from the caller. It has no
  `Default`. `Retry-After` replaces the fallback within the maximum.
  Rejected: invented retry or backoff defaults, because the user has not
  settled the default retry budget.
- **Error payload storage (2026-10-21).** `ExecutionFailed` stores a boxed
  typed `ExecutionError`. Provider text stays available in fields, while
  error diagnostics hide it. Model validation errors also hide nested
  internal text. Rejected: an inline execution payload or raw source
  diagnostics, because they enlarge every result or can expose credentials.
- **Pagination stop data (2026-10-21).** The bounded paginator returns
  records, page count, stop reason, and a resumption request at the page
  limit. A stuck cursor stops traversal with a warning. Rejected: silently
  truncating records without resumption state or repeatedly fetching a stuck
  cursor, because the host needs a bounded, recoverable traversal.

- **Trigger callback admission and cleanup (2026-10-21).** Trigger webhook
  requests accept HTTPS and loopback HTTP URLs, and reject credentials,
  fragments, controls, and other schemes before authentication. A delete with
  `ignore_hook_errors=true` treats a missing deployment as success. Rejected:
  arbitrary callback URLs and unconditional 404 success. Reason: callback
  admission follows the proven Firna boundary, and the public request model
  exposes no separate missing-resource flag.

- **Superseded callback admission (2026-10-22).** The Milestone 6 to 8
  fix round removed the callback URL scheme rule of the decision above,
  because a new rejection needs a user decision. Callback URLs pass
  unchanged. The rule is an open decision. Final review finding 10.
- **Review fix round, failed refresh sharing (2026-10-21).** A generation
  identifies each completed refresh. Queued callers share its typed failure.
  Internal failures retain the shared original source. A later call can
  refresh immediately. Rejected: one request per failed waiter or a default
  hold-off interval. Reason: a burst must not multiply token requests, and
  the plan defines no hold-off default.
- **Review fix round, retry limits (2026-10-21).** A valid `Retry-After`
  above the policy maximum stops retries and returns the mapped error.
  Fallback delays remain capped. Rejected: shortening the server delay,
  because that sends a request before the server permits it.
- **Review fix round, proxy dispatch (2026-10-21).** Pass-through mode checks
  the structured execution envelope, then preserves every status, header,
  and body byte without retries. It invalidates the bearer on any 401.
  `ResponseMode` and `ApiPath` are non-exhaustive. Rejected: normal status
  mapping or keeping a bearer after an upstream 401. Reason: proxy results
  must remain lossless, and Connect and upstream 401s cannot be distinguished.
  The milestone assignment explicitly authorizes this boundary choice.
- **Review fix round, failed page recovery (2026-10-21).** Pagination errors
  carry prior records, completed page count, failed request, and the original
  typed error. Normal `?` propagation still fails the traversal. Rejected:
  dropping prior records or returning a partial success, because the host
  needs both failure propagation and explicit recovery.
- **Review fix round, runtime composition (2026-10-21).** The injected
  constructor requires a sleeper. Only the builder constructs `TokioSleeper`.
  The clock supplies monotonic time for token deadlines and wall time for
  HTTP dates. Rejected: an implicit Tokio sleeper or wall-clock expiry.
  Reason: injected clients must work without Tokio, and wall-clock steps
  must not alter a token lifetime. The milestone assignment authorizes the
  constructor change.
- **Review fix round, request validation (2026-10-21).** JSON construction
  requires the client crate's `Validate` trait and calls the model's method.
  Each request model implements the trait. Rejected: a separate validation
  result, because a resource could supply `Ok(())` without checking its model.

- **Registry query models (2026-10-21).** Actions and registry triggers use
  `ListRegistryRequest`. Only the generic component query carries
  `component_type`. Rejected: reusing that field on routes that do not
  document it, or silently ignoring it. Reason: each typed query must send
  every input that its operation supports without an unsupported field.
- **Registry response envelopes (2026-10-21).** App and component retrieval
  return the complete typed `Data<T>` envelope after checking the returned
  identity. Category retrieval keeps its documented bare response. Rejected:
  returning only the inner app or component, because that drops unknown
  fields on the response envelope.
- **Managed-auth account ownership (2026-10-21).** Account handles offer an
  `AccountRef` path that checks the external-user partition before deletion,
  while the OpenAPI-shaped methods remain available for project-scoped calls.
  Rejected: deleting through an account ID without an optional ownership
  check, because hosts need a safe tenant boundary for cleanup.
- **Managed-auth credential defaults (2026-10-21).** Account list, account
  retrieval, and user-account list requests send `include_credentials=false`
  when callers omit the option. Rejected: relying on a provider default,
  because diagnostics must keep credential retrieval explicit.
- **Proxy resource model (2026-10-21).** The client crate owns `ProxyRequest`
  and returns the existing bounded `HttpResponse` as `ProxyResponse`, with
  `HeaderMap` and byte vectors at the transport boundary. Rejected: a JSON
  proxy model or a filtered response-header map, because proxy calls are
  lossless and may return binary data and repeated headers.
- **Proxy target admission (2026-10-21).** Proxy targets accept absolute
  HTTPS URLs or relative paths and reject credentials, fragments, controls,
  and non-HTTPS absolute URLs before authentication. Rejected: accepting all
  strings or requiring absolute URLs, because the protocol permits app-relative
  paths and URL data must not alter the request origin.
- **User-account list response (2026-10-21).** `UsersApi::list_accounts`
  returns the bare account array defined by the OpenAPI operation. Rejected:
  wrapping it in `Page<Account>`, because that operation has no cursor query
  or page metadata. The accounts protocol records this provider exception.
- **Action blocking field (2026-10-21).** The milestone assignment requires
  action runs with an optional blocking flag. `RunActionRequest` therefore
  carries `blocking` in the JSON body, like configure and reload requests.
  An explicit false value selects the existing nonblocking 202 contract.
  Rejected: omitting the required SDK option or putting it in an undocumented
  query parameter. The current OpenAPI schema and public run-action page
  omit this field. Real-project verification remains blocked on credentials.
- **Review fix, action run contract (2026-10-21).** Action runs follow the
  OpenAPI request model and do not send a blocking flag. Every action-run 202
  is accepted. Rejected: retaining the SDK-only flag, because the public
  operation does not define it and would receive an undocumented body field.
- **Review fix, category listing (2026-10-21).** Category listing returns a
  bare `Vec<AppCategory>` without query parameters. Rejected: wrapping it in
  `Page<AppCategory>`, because the OpenAPI response and public page define a
  bare array.
- **Review fix, query encoding (2026-10-21).** One serde-driven encoder
  validates every query model and emits scalar pairs or repeated array keys.
  Nested objects fail as internal request-shape errors. Rejected: hand-written
  field encoders, because they bypass model validation and drift between
  operations.
- **Review fix, execution status (2026-10-21).** `ExecutionFailed` carries
  the HTTP status and only applies when `error` contains `attribution` or
  `name`. Other error objects use normal status mapping. Rejected: treating
  every object as execution data, because that loses 401, 404, and 429
  categories and their retry metadata.
- **Review fix, resource operation pattern (2026-10-21).** Resource builders,
  JSON dispatch, and identity checks use one shared operation module. A
  filtered page source adapts every paged list to `BoundedPaginator`, and
  `ConnectResources` exposes all resource handles through one mockable factory.
  Rejected: resource-local helpers, because later milestones would repeat the
  same mechanics and drift.
- **Review fix, live test opt-in (2026-10-21).** Credential-gated live tests
  also require `PIPEDREAM_LIVE_TESTS=1`. Rejected: running whenever secrets
  exist, because a complete gate must not make an external API call by
  accident.
- **Second review, nullable response fields (2026-10-21).** Optional response
  fields omit absent values when their schemas do not permit null. Nullable
  fields keep explicit null output. Strict conformance comparisons build
  expected null defaults from nullable schema fields only. Rejected: accepting
  every extra null in the comparison. Reason: that masks null output that a
  schema forbids.
- **Second review, delivery decoding (2026-10-21).** One private helper parses
  trigger deliveries and checks context IDs for both decoding paths. Named
  decode methods hash exact body bytes. Direct serde decoding hashes the JSON
  value text, which excludes outer whitespace. Invalid JSON retains its body
  error. Invalid delivery shapes construct `InvalidTriggerDelivery`. Rejected:
  a check only in the named methods or deleting the unused shape error.
  Reason: both paths need validation, and callers can distinguish bad shapes.
- **Second review, JSON errors and numbers (2026-10-21).** Raw JSON access
  returns `PayloadError::Json` with the original serde error as its source.
  Timer interface intervals use `serde_json::Number` to retain integer values.
  Shared models keep exact integers only within the `i64` and `u64` ranges.
  Larger integers and long decimals can round. Exact number text stays in raw
  webhook fields. Rejected: public serde errors and floating-point intervals.
  Reason: crate errors keep one boundary, and integer schedules must survive.
- **Second review, request validation (2026-10-21).** Token builders, explicit
  validation, and serde share one lifetime validator. Trigger event serde uses
  the request constructor. Rejected: separate range checks for serde.
  Reason: shared checks keep construction and deserialization aligned without
  adding validation beyond the assigned range checks.
- **Administration response forms (2026-10-21).** File Stash downloads return
  a `FileStashDownload` enum with bytes or a redirect location, and workflow
  calls return the shared pass-through HTTP response. Rejected: decoding both
  responses as JSON, because file contents and workflow statuses are not
  limited to JSON.
- **Workflow method and target validation (2026-10-21).** Workflow calls use
  `http::Method`, accept only GET, POST, PUT, DELETE, and PATCH, and validate
  an absolute HTTPS target before bearer acquisition. Rejected: accepting any
  HTTP method or building a project-relative URL, because the documented
  workflow endpoint is an HTTPS URL with a fixed method set.
- **Runtime rate-limit token selection (2026-10-21).** The client exposes a
  `with_rate_limit_token` method in addition to builder configuration, so a
  token returned by `RateLimitsApi` can be applied to a cloned client. Rejected:
  mutating a live client configuration in place, because client settings are
  immutable after construction.
- **Review fixes for managed auth and proxy (2026-10-22).** Named proxy
  methods overwrite a mismatched constructor method, and proxy bodies move
  into the request without a second copy. Account ownership mismatches map to
  `NotFound` and account creation carries typed query filters. Token bodies
  always carry the configured project. Rejected: trusting caller method or
  request project fields, because those values can change the operation route
  or tenant.
- **Review fixes for response modes (2026-10-22).** Proxy pass-through checks
  only non-2xx bodies with `attribution`, while workflow pass-through returns
  every status, header, and byte without an execution check. Rejected: one
  shared execution check, because workflows and upstream proxy responses have
  different lossless contracts.
- **Review fixes for trigger lifecycle (2026-10-22).** Read identity checks
  use one helper, while deploy and update responses remain unchanged. Missing
  deployment tolerance is a separate delete choice from `ignore_hook_errors`.
  Callback URLs keep their pre-Milestone-7 provider-defined behavior.
  Rejected: checking mutation response identities or coupling missing deletes
  to hook delivery, because the API can return a new identity and the flags
  control different concerns.
- **Review fixes for URL and diagnostic boundaries (2026-10-22).** API origin,
  workflow, and token URI validation use one helper with explicit rule sets.
  File Stash debug output reports byte counts and query-free redirect URLs.
  Rejected: repeating URL predicates or printing presigned query values,
  because duplicated rules drift and presigned queries are credentials.

- **CLI settings (2026-10-21).** Each connection flag takes precedence over
  its `PIPEDREAM_*` variable. An injected reader resolves the variables after
  parsing. An empty variable counts as unset. One message names every missing
  required setting. `PIPEDREAM_API_ORIGIN` is the optional origin variable.
  The environment has no default. Rejected: clap's `env` attribute, because
  it reads the process environment during parsing, which unit tests must not
  touch. Rejected: a development default, because the client protocol keeps
  the environment explicit.
- **CLI pipeline (2026-10-21).** The CLI parses arguments, resolves settings,
  validates the command, and reads its input files. Then it builds the client
  once and sends the request. Rejected: building the client first, because a
  validation failure must exit before any client or request exists.
- **CLI output and redaction (2026-10-21).** A successful command prints one
  pretty JSON document that is encoded before any write. Every printed model
  implements a `Redact` trait that replaces its credential-class fields.
  `--show-secrets` skips it. A command that exists to produce a credential
  uses an explicit revealed document. An accepted action run prints
  `{"status": "accepted"}` and exits with code 0. Rejected: JSON-path
  redaction, because it does not follow model changes. Rejected: a failure
  exit for an accepted run, because the protocol defines it as a result.
- **CLI failures (2026-10-21).** The exit codes are 0 success, 1 internal,
  2 usage or validation, 3 authentication or access, 4 not found, 5 rate
  limited, 6 unavailable, and 7 rejected request or failed action. Each
  message names the failure, the flag or variable, and one recovery action.
  Messages never print provider messages, provider codes, or bodies. JSON
  input errors keep only the line and the column. Rejected: provider text in
  errors, because SDK diagnostics hide it, as it can echo credentials.
- **CLI pages (2026-10-21).** Paged list commands take `--limit`, `--after`,
  `--before`, and `--all` with a `--max-pages` budget of 10 by default.
  `--all` prints the records, the page count, the stop reason, and the next
  request. A failed page prints no records and names the resume cursor.
  `categories list` takes no page flags, because Pipedream returns a bare
  array. Rejected: an unbounded `--all`, because a cursor fault could exhaust
  memory. Rejected: a required budget flag, because it burdens every call.
- **CLI command shape (2026-10-21).** A command that reads or runs one app or
  registry component takes its key as a positional argument: `apps get <APP>`,
  `components get <COMPONENT>`, and `actions run <ACTION>`. Rejected: an
  `--action` flag, because one positional rule keeps these commands
  consistent.
- **CLI review fix, uncertain mutations (2026-10-21).** For a command that
  changes data, an unavailable, oversized, or unreadable response replaces
  the recovery action with a warning that the change may have happened.
  Oversized responses exit 1, like other unreadable responses. Rejected: a
  warning only for unavailable responses, because the request also left in
  the other two cases.
- **CLI review fix, failure text (2026-10-21).** A network failure without
  an HTTP status names the API origin. A 408 or 5xx names the origin and its
  status, because the CLI did reach it. An interrupted listing gives one
  action: the resume command for a retryable failure, else its own action.
  Resume hints quote the cursor for a POSIX shell. Rejected: two recovery
  actions in one message.
- **CLI review fix, inputs (2026-10-21).** Page sizes and budgets are text in
  clap. A bounded number that does not parse becomes zero, so the SDK rule
  reports its documented range. `--max-pages` has no upper limit. Props JSON
  that nests deeper than the decoder's 127 levels gets its own message before
  decoding. Rejected: clap number parsers, because their errors do not follow
  the CLI message style.
- **CLI trigger deployment shape (2026-10-21).** `triggers deploy` takes the
  component key as a positional argument, like `actions run`. This replaces
  the `--component` flag of the CLI sketch. Rejected: the sketch flag,
  because one positional rule keeps the component commands consistent.
  Settled by the orchestrator.
- **CLI lists without cursors (2026-10-21).** `triggers events` maps
  `--limit` to the API count `n` and sends the Pipedream default of 20 when
  the flag is absent. `usage` takes `--start` and `--end`. Neither takes
  cursor flags, the same exception as `categories list`. `usage` accepts Unix
  seconds or RFC 3339 times. Rejected: cursor flags that the API ignores, and
  Unix seconds only, because RFC 3339 times are easier to type.
- **CLI proxy results (2026-10-21).** A proxy command prints the upstream
  status, every header, and the body. Valid UTF-8 prints as text and other
  bytes as base64, with an `encoding` field. An upstream status of 400 or
  more exits 0, like `curl` without `--fail`. Rejected: mapping upstream
  statuses to exit codes, because the proxy call succeeded and the client
  cannot tell an upstream 401 from a Connect 401. Settled by the orchestrator.
- **CLI proxy headers (2026-10-21).** The CLI parses each `--header` and
  checks a copy of the SDK reserved-name list before any request, so an
  error can name the header. Errors never show a header value. A test runs
  every reserved name through the SDK proxy handle to catch drift. Rejected:
  the SDK error alone, because it does not name the header.
- **CLI account reads (2026-10-21).** `accounts get` uses the SDK ownership
  check when the caller passes `--external-user-id`. The CLI no longer keeps
  redaction for models that no command prints. Rejected: keeping those
  implementations, because no command reaches them.
- **CLI review fix, settings (2026-10-21).** The client secret flag parses
  into `SecretString`. The credential flags take no hyphen values, so a
  following flag is never taken as a credential. A custom API origin gets one
  note before any request. Rejected: hyphen values for credentials, because
  `--client-secret --show-secrets` silently used the flag name as the secret.
- **CLI adaptation to the SDK review fixes (2026-10-22).** `triggers delete`
  takes `--missing-ok`, which sets the SDK choice that accepts a missing
  deployment. Without the flag, a missing trigger exits 4. The CLI sends
  `--webhook-url` to Pipedream unchanged, because the SDK removed its local
  callback URL rule. Trigger IDs take the SDK prefixes `dc_`, `hi_`, and
  `ti_`. An account of another external user exits 4 through the SDK
  `NotFound` mapping. Rejected: accepting a missing trigger by default,
  because a delete with a wrong ID would then succeed silently. Rejected: a
  CLI copy of the removed URL rule, because the SDK no longer has that
  policy.

## Open decisions

The user answered every item below on 2026-10-26 except where an item says
otherwise. Milestones 13 and 14 apply the outcomes. Decision numbers 66 to 76
and finding numbers come from the final review report of 2026-10-22.

- **GET retry defaults (2026-10-21).** Decision 4 requires an opt-in GET
  retry policy but does not settle the default total attempts or maximum
  delay. **A.** Three total attempts, with a 30-second maximum delay and
  `Retry-After` support. Effort: Medium. **B.** Two total attempts, with a
  60-second maximum delay and `Retry-After` support. Effort: Medium.
  **Recommendation:** A, because it matches the settled two-retry wording
  while bounding delay and work. The protocol leaves this default open.
  **Outcome (decision 67):** A: three attempts and a 30-second maximum delay,
  chosen by the user on 2026-10-26.
- **Webhook body limit (2026-10-21).** Milestone 9 requires a bounded webhook
  body but does not settle its default. **A.** Use a 1 MiB default, with a
  configurable limit. Effort: Small. **B.** Use the 10 MiB client response
  default, with a configurable limit. Effort: Small. **Recommendation:** A,
  because webhook ingress should have a smaller independent bound. The
  protocol leaves this default open.
  **Outcome (decision 68):** B: a 10 MiB default, configurable, chosen by the
  user on 2026-10-26.

- **Literal dot path parameters (2026-10-21).** The URL library skips `.`
  and `..` when appending path segments. Some generic identifiers currently
  accept those values. **A.** Reject literal dot identifiers before a token
  call and align the types grammar. Effort: Medium. **B.** Preserve literal
  dot parameters through an HTTP URL representation that does not normalize
  them. Effort: Large. **Recommendation:** A, because it prevents a route
  change with one identifier rule. No new rejection is applied before the
  user answers. This choice blocks final handling of these identifiers in
    resource paths.
  **Outcome (decision 66):** A: reject `.` and `..` identifiers before the
  token call, chosen by the user on 2026-10-26.

- **CLI contract home (2026-10-21).** The CLI exit codes, settings, output,
  and redaction rules are requirements at the CLI boundary. They live in the
  CLI README and in the plan decisions. No protocol doc holds them. **A.** Keep
  them in the CLI README. Effort: Small. **B.** Add `docs/protocol/cli.md`
  with the CLI requirements and the decisions in force, and link it from the
  CLI README. Effort: Large. **Recommendation:** B, because the post-push
  review checks behaviour against protocol docs. No protocol doc is added
  before the user answers.
  **Outcome (decision 74):** A: keep the CLI rules in the CLI README, chosen
  by the user on 2026-10-26.
- **CLI redaction mechanism (2026-10-21).** Each printed model lists its
  credential-class fields in a `Redact` implementation. A new credential
  field in an existing model stays unredacted until someone adds it to the
  list, and an unknown provider field prints as it is. **A.** Keep the field
  lists. Effort: Small. **B.** Match every model field exhaustively, so a new
  field fails the build until someone classifies it. Effort: Medium. **C.**
  Derive redaction from the secret wrapper types in the types crate, so every
  `SecretString`, `SecretJson`, and `ConnectToken` redacts itself. Effort:
  Large. **Recommendation:** B, because a new model field then fails the CLI
  build until someone classifies it, without a change to the types crate. No
  option is applied before the user answers.
  **Outcome (decision 69):** A: keep the per-model field lists, chosen by the
  user on 2026-10-26.

- **Workflow host allow-list (2026-10-22).** Workflow invocation sends the
  project bearer to any HTTPS URL that the caller passes. The official
  Pipedream SDK accepts only hosts under its workflow domain. **A.** Keep any
  HTTPS host and document the risk. Effort: Small. **B.** Add a configurable
  workflow domain list with `m.pipedream.net` as the default, checked before
  the token call. Effort: Medium. **C.** B, and make absolute targets
  constructible only inside the crate. Effort: Medium. **Recommendation:** C,
  because it keeps the bearer on Pipedream hosts and removes the generic
  path. Final review finding 1, severity High.
  **Outcome (finding 1):** A: keep any HTTPS host, and document the risk,
  chosen by the user on 2026-10-26.
- **Proxy target backslash and raw `@` (2026-10-22).** A relative proxy target
  such as `/\evil.example/api` passes the origin check, and WHATWG URL
  resolution turns it into another host. **A.** Keep the current rules.
  Effort: Small. **B.** Reject a backslash anywhere in a proxy target and a
  raw `@` in an absolute target's authority. Effort: Small. **C.** B, through
  the shared URL rule helper. Effort: Large. **Recommendation:** B. Final
  review finding 4.
  **Outcome (finding 4):** C: reject them through the shared URL rule helper,
  chosen by the user on 2026-10-26.
- **Cursor grammar (2026-10-21).** Cursors use the common identifier grammar:
  at most 250 bytes and no `/`, `\`, `?`, or `#`. The Firna adapter accepted
  visible ASCII up to 2 KiB. A real cursor outside the grammar fails the whole
  page. **A.** Keep the common grammar. Effort: Small. **B.** Accept visible
  ASCII up to 2 KiB for cursors, which go into query strings. Effort: Small.
  **Recommendation:** B, because cursors are opaque and never enter a path.
  **Outcome (decision 70):** B: visible ASCII up to 2 KiB, chosen by the user
  on 2026-10-26.
- **`appcat_` prefix (2026-10-21).** `AppCategoryId` requires `appcat_`. The
  OpenAPI document gives `AppCategory.id` no pattern, and the retrieve path
  uses `^appcar_`. **A.** Keep `appcat_`. Effort: Small. **B.** Use the common
  grammar for category IDs. Effort: Small. **C.** Decide after one live
  category list. Effort: Small. **Recommendation:** C, then A or B.
  **Outcome (decision 71):** A: keep `appcat_`, chosen by the user on
  2026-10-26.
- **Trigger callback URL rule (2026-10-22).** Milestone 7 added an HTTPS or
  loopback-HTTP rule for trigger callback URLs without a user decision. The
  fix round removed it, so callback URLs pass unchanged. **A.** No rule.
  Effort: Small. **B.** Require HTTPS, as for Connect token URIs. Effort:
  Small. **Recommendation:** B, because Pipedream cannot reach loopback URLs.
  **Outcome (decision 72):** A: no rule, chosen by the user on 2026-10-26.
- **202 for a blocking request (2026-10-21).** A 202 from configure or
  reload is `Accepted` when `blocking` is omitted or false. A 202 for an
  explicit `blocking: true` decodes its body. **A.** Keep it. Effort: Small.
  **B.** Map every 202 in execution mode to `Accepted`. Effort: Medium.
  **Recommendation:** B, because a 202 has no result body.
  **Outcome (decision 73):** B: every 202 in execution mode is `Accepted`,
  chosen by the user on 2026-10-26.
- **Unknown-field redaction (2026-10-22).** `Emitter::Other` and the
  unknown-field maps print raw JSON in `Debug`, so a credential field that
  Pipedream adds later prints in diagnostics. **A.** Keep decision 5:
  credential-class fields only. Effort: Small. **B.** Mask unknown fields
  whose names look like secrets. Effort: Medium. **Recommendation:** B, with
  a fixed name list. Final review finding 50.
  **Outcome (finding 50):** A: keep decision 5, chosen by the user on
  2026-10-26.

## Public API sketch

```rust
let client = PipedreamClient::builder()
    .project_id(ProjectId::new("proj_abc123")?)
    .environment(Environment::Development)
    .client_credentials(ClientCredentials::new(client_id, client_secret)?)
    .build()?;
let page = client.apps().list(ListAppsRequest::default()).await?;
let next = PageRequest::default().next(&page.page_info)?;
let token = client
    .tokens()
    .create(CreateConnectTokenRequest::new(external_user_id))
    .await?;
let response = client
    .proxy()
    .send(ProxyRequest::get(account, "https://slack.com/api/conversations.list"))
    .await?;
```

`PipedreamClient::new(config, transport, clock, sleeper)` takes shared
`HttpTransport`, `Clock`, and `Sleeper` trait objects for composition roots
and tests. The builder constructs the reqwest transport, the system clock,
and `TokioSleeper`. The runtime-composition decision on 2026-10-21 records
this constructor change.

The CLI sketch, with credentials from the `PIPEDREAM_*` variables:

```sh
pipedream-cli apps list --query calendar
pipedream-cli components list --app google_calendar --type action
pipedream-cli tokens create --external-user-id user_1
pipedream-cli proxy get --external-user-id user_1 --account apn_x URL
pipedream-cli triggers deploy google_calendar-new-or-updated-event-instant \
  --external-user-id user_1 --props props.json \
  --webhook-url https://example.com/hook
```

The public error contract of the client crate:

- `InvalidRequest(InvalidRequest)`: local validation failed on an
  identifier, a URL, the header policy, or a limit. No request was sent.
- `Unauthorized`, `Forbidden`, `NotFound`: a 401, 403, or 404 with the
  envelope message.
- `RateLimited { retry_after, limit, remaining, reset }`: a 429 with the
  parsed headers.
- `Rejected { status, code, message }`: any other 4xx.
- `ExecutionFailed { status, error }`: structured attribution and HTTP status
  from an action or proxy run.
- `Accepted`: a 202 from configure or props without blocking, or any action
  run 202.
- `Unavailable { status }`: a 408, a 5xx, a timeout, or a transport
  failure. The request may have run.
- `ResponseTooLarge { limit }`: the body passed the configured cap.
- `Internal(InternalError)`: an encode, decode, or client construction
  failure.

## Milestone 1: Documentation and protocol

Summary: write the requirements and the decisions before any code, with the
recommended options applied.

- [x] Write `docs/protocol/connect-client.md`:
  - [x] OAuth client-credentials token acquisition, the one-hour lifetime,
        and early refresh.
  - [x] The `x-pd-environment` header, the optional `x-pd-rate-limit`
        header, and the other request headers.
  - [x] Status mapping to the error contract, `Retry-After` and
        `X-RateLimit-*` parsing.
  - [x] Response caps, timeouts, the no-replay rule, and the opt-in GET
        retry policy.
  - [x] The redaction rule for credential-class values.
- [x] Write `docs/protocol/connect-resources.md`:
  - [x] The covered operations grouped by resource.
  - [x] Cursor pagination with `limit`, `after`, and `before`.
  - [x] Lossless models and identifier grammars.
  - [x] Connect token limits and `include_credentials`.
  - [x] Proxy encoding and header policy, and the 202 contract.
  - [x] The trigger lifecycle operations.
- [x] Write `docs/protocol/webhooks.md`:
  - [x] `x-pd-signature` verification, the 300-second default tolerance,
        and the body cap.
  - [x] Connection event payloads and trigger delivery payloads.
  - [x] Key selection between trigger webhooks and the project environment
        webhook.
- [x] Write `docs/architecture.md`: the three library crates, the CLI, the
      transport and clock boundaries, the token provider, and the host
      responsibilities.
- [x] Write the workspace `README.md`: features, quick start, the CLI
      commands, developer get-started, key code, and links to the docs and
      `plans/`.
- [x] Validate the Markdown, review the diff, commit, and push.

Evidence: `.context/pipedream-connect-sdk/markdownlint.txt` and
`.context/pipedream-connect-sdk/operation-table-check.txt`.

## Milestone 2: Repository bootstrap

Summary: a compiling workspace with the gate and CI, no SDK code yet.

- [x] Create the Cargo workspace with `crates/pipedream-connect-types`,
      `crates/pipedream-connect`, `crates/pipedream-connect-webhooks`, and
      `xtask`. Give each a `README.md` in the required section order.
      Milestone 10 adds `crates/pipedream-cli`.
- [x] Add `rust-toolchain.toml`, `.cargo/config.toml` with the `xtask`
      alias, `LICENSE`, and `.gitignore` entries for `target` and `.context/`.
- [x] Add `internal-error` as a workspace git dependency at revision
      `6477922`, and the shared external dependencies with `cargo add`.
- [x] Implement `cargo xtask check` with these steps:
  - [x] `cargo fmt --all -- --check`.
  - [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
  - [x] `cargo test --workspace --all-features`.
  - [x] `cargo doc --no-deps` with warnings denied.
  - [x] A Rust file-length lint with the 300-line cap.
- [x] Add `.github/workflows/ci.yml` that runs the same steps on pull
      requests and on `main`.
- [x] Create `AGENTS.md` from the shared template: commands, repository
      rules, review focus, and status. Add `CLAUDE.md` as a symlink to it.
      Milestone 10 adds the CLI smoke command.
- [x] Run the gate, commit, and push.
      Evidence: `.context/pipedream-connect-sdk/bootstrap-gate.txt`.

## Milestone 3: Typed models

Summary: the `pipedream-connect-types` crate, pure data with serde.

- [x] Identifiers with one grammar for construction and deserialization:
      `ProjectId`, `ExternalUserId`, `AccountId`, `DeployedTriggerId`,
      `OauthAppId`, `AppCategoryId`, `AppSlug`, `ComponentKey`, `PropName`,
      `WebhookId`, `ConnectToken`, and `Cursor`.
- [x] `SecretString` and `SecretJson` with redacted `Debug` and explicit
      exposure, and `Environment`.
- [x] Pagination: `PageRequest` with limit 1 to 200, `PageInfo`, `Page<T>`,
      and `PageRequest::next` that detects a stuck cursor.
- [x] Registry models: `App`, `AppAuthType`, `ProxySupport`, `AppCategory`,
      `ListAppsRequest` with sort and filters, `Component`, `ComponentType`,
      `ToolAnnotations`, `ComponentStash`, and `ListComponentsRequest`.
- [x] Prop models:
  - [x] `ConfigurableProp` with the shared base fields and a lossless
        `PropKind`.
  - [x] `PropOption` for labelled, nested, and scalar choices, and
        `TimerValue`.
  - [x] `ConfiguredProps` with a typed `ConfiguredPropValue` that keeps
        app, SQL, and raw JSON values.
- [x] Managed-auth models: `Account` with the nested `App` and optional
      credentials, `CreateAccountRequest`, `ListAccountsRequest`,
      `CreateConnectTokenRequest` with `expires_in` 1 to 14,400,
      `ConnectTokenResponse`, `ValidateTokenResponse`, and `ExternalUser`.
- [x] Execution models: `ConfigurePropRequest` and response,
      `ReloadPropsRequest` and response with `DynamicProps`,
      `RunActionRequest` with `stash_id`, `RunActionResponse`,
      `Observation`, and `ExecutionError` with the attribution origin.
- [x] Trigger models:
  - [x] `DeployTriggerRequest` and the `Emitter` enum for deployed
        components, HTTP interfaces, and timer interfaces.
  - [x] `UpdateTriggerRequest`, `TriggerWebhook`, `TriggerWebhooks`, and
        `TriggerWorkflows`.
  - [x] `EmittedEvent` and `ListDeployedTriggersRequest` with
        `emitter_type`.
- [x] Administration models: `Project`, `ProjectInfo`, `OauthApp`,
      `AppOverride`, `ProjectWebhook`, `UsageRecord`, `RateLimitRequest`,
      `RateLimitToken`, and the create and update requests.
- [x] Wire envelopes: `Data<T>`, `PageEnvelope<T>`, and `ErrorResponse`.
- [x] Vendor the OpenAPI document under the crate `tests/fixtures/` with a
      refresh script. Add a conformance test that decodes every response
      example in it into the matching model.
- [x] Unit tests for every grammar, lossless decoding of unknown kinds,
      redacted `Debug`, and pagination. Write the crate README.
- [x] Run the gate, commit, and push.
      Evidence: `.context/pipedream-connect-sdk/typed-models-gate-final.txt`.

- [x] Review fix round for Milestones 3 and 9: resolve review items 1 to 36,
      update the docs, run the gate, commit, and push.
      Evidence: `.context/pipedream-connect-sdk/review-targeted-tests.txt` and
      `.context/pipedream-connect-sdk/review-gate.txt`.

- [x] Second review fixes: response nullability, conformance samples, dead
      timestamp adapter, number precision docs, timer intervals, and request
      deserialization checks. Run targeted tests and the complete gate.
      Validate Markdown and commit without a push for the orchestrator.
      Evidence: `.context/review-m3-fix2/targeted-final.txt`,
      `.context/review-m3-fix2/gate.txt`, and
      `.context/review-m3-fix2/markdown-final.txt`.

## Milestone 4: Client core

Summary: the `pipedream-connect` crate can authenticate and dispatch, with
no resource yet.

- [x] `ClientConfig` and the builder:
  - [x] HTTPS origin validation, project, and environment.
  - [x] Credentials or a static bearer, scope, and the rate-limit token.
  - [x] Timeouts, the response cap, and the retry policy.
- [x] `Clock` trait with `SystemClock`, and `HttpTransport` with redacted
      `HttpRequest` and `HttpResponse` types.
- [x] `ReqwestTransport` behind the `reqwest` feature: rustls, no redirects,
      no client retries, configured timeouts, and a streamed body that stops
      at the cap.
- [x] `AccessTokenProvider` trait with the client-credentials provider and
      the static provider, including early refresh, single flight, and
      invalidation on 401.
- [x] Shared dispatch:
  - [x] URL building under `/v1/connect/{project_id}` and the headers.
  - [x] Status mapping to the error contract, `Retry-After` and
        `X-RateLimit-*` parsing, and the execution envelope.
  - [x] Bounded JSON decoding.
- [x] Opt-in `RetryPolicy` for GET requests with an injected `Sleeper`,
      honouring `Retry-After` and a maximum delay.
- [x] A bounded `paginate` helper that drives `PageRequest::next` up to a
      page limit.
- [x] Unit tests with a scripted unimock transport and clock:
  - [x] Concurrent refresh, early refresh, and 401 invalidation without
        replay.
  - [x] Every status mapping and the rate-limit header parsing.
  - [x] Cap enforcement and retry counts checked by recorded requests.
- [x] A local-socket integration test for the reqwest transport: exact
      headers, announced and streamed oversize bodies, and a redirect that
      is not followed. Write the crate README.
- [ ] Resolve literal dot path parameters after the user decides.
- [x] Run the gate, commit, and hand off to the orchestrator. The assigned
      worktree branch must not push. The orchestrator merges and pushes it.

Evidence: `.context/client-core/` contains the targeted test results, smoke
results, regression evidence, Markdown checks, and the complete gate.

- [x] Resolve Milestone 4 review items 1 to 14 with regression tests,
      aligned docs, the complete gate, separate fix and test commits,
      and a push. Evidence: `.context/client-review-fixes/`.
- [x] Preserve custom-transport validation errors in shared refresh failures
      and show safe model field bounds in diagnostics. Added while checking
      Milestone 5. Evidence: `.context/registry-resources/`.

## Milestone 5: Registry and execution resources

Summary: apps, components, actions, and registry triggers work against the
real API.

- [x] `AppsApi`: list with every filter, get by slug with an identity check.
- [x] `AppCategoriesApi`: list and get.
  - [x] Retrieve a category with an identity check.
  - [x] Resolve the category-list response-shape decision, then implement
        and test the chosen contract.
- [x] `ComponentsApi`: list with `component_type`, get with `version`,
      configure a prop, and reload props.
- [x] `ActionsApi`: list, get, configure, reload props, and run with
      `blocking`, `stash_id`, and the 202 contract.
- [x] `TriggersApi` for registry triggers: list, get, configure, and reload
      props.
- [x] Unit tests per resource: query encoding, request bodies, response
      mapping, lossless unknown kinds, and error mapping.
- [x] A credential-gated live test that reads the `google_calendar` app and
      lists its actions and triggers in a development project. It skips
      without `PIPEDREAM_CLIENT_ID`, `PIPEDREAM_CLIENT_SECRET`, and
      `PIPEDREAM_PROJECT_ID`.
- [x] Update the READMEs and the resources protocol doc only if a contract
      changed.
- [x] Run the gate, commit, and push the independent resource work.
- [ ] Run the live test against a development project. Blocked: the three
      Pipedream credential variables are absent.

Evidence: `.context/registry-resources/` holds targeted checks, the live-test
skip, documentation validation, and contract notes.

## Milestone 6: Managed auth and proxy resources

Summary: tokens, accounts, users, and the proxy work against the real API.

- [x] `TokensApi`: create with URL validation of the redirect and webhook
      URIs, and validate.
- [x] `AccountsApi`: list with filters, get with and without credentials,
      create from a Connect token, and delete. Delete all accounts of an app
      for the project.
- [x] `UsersApi`: list, delete, and list the accounts of one external user.
- [x] `ProxyApi`: GET, POST, PUT, PATCH, and DELETE with the header policy,
      the base64 target, byte bodies, and the lossless response.
- [x] `ProjectsApi::info` for the apps enabled in the project environment.
- [x] Correction (2026-10-21): `ProjectsApi::info` moved to Milestone 8, so
      that one agent owned `ProjectsApi`. Milestone 8 implemented it.
- [x] Unit tests: ownership checks, `include_credentials` handling with
      redacted `Debug`, every blocked header, prefixing, target encoding,
      and upstream error statuses preserved.
- [x] Credential-gated live tests cover token create and validation, account
      listing, and proxy GET and POST. They use the live-test opt-in and a
      short-lived token.
- [x] Update the READMEs and the resources protocol doc only if a contract
      changed.
- [x] Run the gate and commit. The assigned worktree does not push.

## Milestone 7: Trigger lifecycle resources

Summary: deploy and manage triggers for an external user.

- [x] `TriggersApi::deploy` with `webhook_url`, `workflow_id`,
      `emit_on_deploy`, and the `Emitter` response.
- [x] `DeployedTriggersApi`:
  - [x] List with `emitter_type`, get, update, and delete with
        `ignore_hook_errors`.
  - [x] List events with `n`, and get and set workflows.
  - [x] List and set webhooks, get one webhook, and regenerate its signing
        key.
- [x] Unit tests: tenant scoping on every call, idempotent delete of a
      missing trigger, epoch-millisecond timestamps, signing-key redaction,
      and the emitter variants.
- [x] A credential-gated live test that deploys a timer-free Google
      Calendar trigger to a loopback webhook URL, lists it, and deletes it.
- [x] Correction (2026-10-22, `a4c09f1`): the trigger has a timer property,
      the app property is `googleCalendar`, and the test always deletes the
      triggers that it created, also after a failed assertion.
- [x] Update the READMEs and the resources protocol doc only if a contract
      changed.
- [x] Run the complete gate and commit the branch without pushing.
- [ ] Run the trigger live test against a development project. Blocked: the
      required Pipedream credential and connected-account variables are absent.

## Milestone 8: Administration resources

Summary: the remaining Connect operations.

- [x] `ProjectsApi`: list, create, get, update, delete, and update logo.
- [x] `OauthAppsApi` and `AppOverridesApi`: list, create, get, update, and
      delete.
- [x] `ProjectWebhooksApi`: get, set, delete, and regenerate the signing
      key for the project environment webhook.
- [x] `UsageApi::list` for a time window, and `RateLimitsApi::create` that
      returns the token for the `x-pd-rate-limit` header.
- [x] `FileStashApi::download` that returns bytes or the redirect location
      without following it.
- [x] `WorkflowsApi::invoke_for_external_user` against a workflow endpoint
      URL with the external user and environment headers.
- [x] A coverage test that reads the vendored OpenAPI document and asserts
      that every `operationId` appears in the SDK coverage table.
- [x] Unit tests per resource. Update the READMEs.
- [x] Run the gate, commit, and hand off. The assigned worktree branch must
      not push. The orchestrator merges and pushes it.

## Milestone 9: Webhook verification

Summary: the `pipedream-connect-webhooks` crate.

- [x] `WebhookSignatureVerifier` trait with the HMAC-SHA256 verifier:
  - [x] Exact `t=<seconds>,v1=<hex>` parsing and constant-time comparison.
  - [x] A configurable tolerance with a 300-second default, a body cap, and
        an explicit receipt time.
- [x] `ConnectionEvent` for `CONNECTION_SUCCESS` and `CONNECTION_ERROR`
      with the account, Connect token, environment, and session id.
- [x] `TriggerDelivery` with the event payload and the context id and
      timestamp, and a fallback event id from the body hash.
- [x] Unit tests with fixed timestamps: valid, stale, future, duplicate
      header parts, bad hex, empty key, and oversize body. Write the crate
      README.
- [x] Run the gate, commit, and push.

- [x] Review fix round for Milestones 3 and 9: resolve review items 37 to 40,
      preserve raw webhook JSON, update the docs, run the gate, commit, and
      push. Leave the webhook body-limit default open.

- [x] Second review fixes: exact body identity, shared context ID validation,
      shape errors, and raw JSON crate errors. Update the README, run targeted
      tests and the complete gate, and commit without a push.
      Evidence: `.context/review-m3-fix2/targeted-final.txt` and
      `.context/review-m3-fix2/gate.txt`.

## Milestone 10: CLI

Tags: ui

Summary: the `pipedream-cli` crate runs every smoke command against the
resource traits.

- [x] Create `crates/pipedream-cli` with `clap`. Mark it `publish = false`.
      Give it a `README.md` in the required section order.
- [x] Shared configuration flags, the `PIPEDREAM_*` variables, JSON output,
      and exit codes.
- [x] Registry and execution commands: `apps list`, `apps get`,
      `categories list`, `components list`, `components get`,
      `actions list`, and `actions run`.
- [x] Managed-auth and proxy commands: `tokens create`, `accounts list`,
      `accounts get`, `users list`, and `proxy <method>`. `tokens create`
      prints the Connect Link URL for an external user.
- [x] Trigger commands: `triggers list`, `triggers get`, `triggers deploy`,
      `triggers delete`, and `triggers events`.
- [x] Administration commands: `project info`, `usage`, and `webhook get`.
- [x] CLI unit tests with mocked resource traits for every command:
      argument parsing, request mapping, JSON output, and exit codes.
  - [x] Registry and execution commands, settings, input files, the
        redaction of every credential-bearing model, failure text, and exit
        codes.
  - [x] Managed-auth, proxy, trigger, and administration commands, with the
        redaction of each response that holds a secret.
  - [x] Marker values in every credential-class field of every printed
        model, and a drift test of the proxy header policy against the SDK.
- [x] The CLI README with the command reference. Add the CLI to the
      workspace README, and the CLI smoke command to `AGENTS.md`.
  - [x] Write the reference with the planned commands marked, the workspace
        README CLI section, and the `AGENTS.md` smoke command.
  - [x] Replace the planned entries when Milestones 6 to 8 merge.
- [x] Add a credential-free smoke: build the binary, print the help of every
      command, and run every command without credentials. Evidence:
      `.context/cli/smoke-phase1.txt` in the CLI worktree.
- [x] Apply the phase 1 review fixes 1 to 9 with tests that fail first.
      Evidence: `.context/cli/review-fixes-red.txt`,
      `.context/cli/secret-debug-red.txt`, and
      `.context/cli/review-fixes-green.txt` in the CLI worktree.
- [ ] Run the gate, run every command against a development project,
      commit, and push.
  - [x] Run the gate and commit the registry and execution commands. The
        assigned worktree branch does not push. The orchestrator merges it.
        Evidence: `.context/cli/gate-phase1.txt` in the CLI worktree.
  - [x] Merge Milestones 6 to 8, add the remaining commands, rerun the gate,
        and commit. Evidence: `.context/cli/gate-phase2.txt`,
        `.context/cli/smoke-phase2.txt`, and `.context/cli/merge-audit.txt`
        in the CLI worktree.
  - [x] Merge the SDK review fix round, adapt the CLI to its changes, rerun
        the gate, and commit. Evidence: `.context/cli/gate-phase3.txt`,
        `.context/cli/smoke-phase3.txt`, and `.context/cli/merge2-audit.txt`
        in the CLI worktree.
  - [ ] Run every command against a development project. Blocked: the
        Pipedream credential variables are absent.

## Milestone 11: Smoke and release readiness

Summary: the SDK and the CLI are usable by a stranger.

- [x] Each library crate `lib.rs` includes its README as crate docs, so the
      Quick Start compiles as a doctest.
- [x] `cargo doc --no-deps` builds without warnings, and `Cargo.toml`
      carries description, repository, keywords, and categories.
- [ ] Run the complete smoke against a development project: every CLI
      command and both live tests. Save the output under
      `.context/pipedream-connect-sdk/smoke.md`.
- [x] Run the credential-free smoke checks and save the output under
      `.context/pipedream-connect-sdk/smoke.md`.
- [ ] Run every CLI command and both live tests against a development project.
      Blocked: the Pipedream credential and development-project variables are
      absent.
- [x] Update the workspace README and the architecture doc for anything
      learnt.
- [x] Run the gate, commit, and push.

## Milestone 12: Verification, commit, and review

Summary: the final checks and the review loop.

- [x] Run the targeted tests and the complete gate named in `AGENTS.md`.
- [x] Commit with Conventional Commits and push the branch.
- [x] Review the complete local diff against `origin/main` with the
      `implementation-review` skill after the push. Report the findings,
      then apply the review-fix rule: fix the `Auto-fix: yes` findings,
      re-review once, and report the rest.
      Evidence: `.context/pipedream-connect-sdk/final-review/report.md`.
  - Finding 1: Workflow calls send the project bearer to any HTTPS host.
    Severity High, effort Medium. Recommended: C, workflow domain allow-list and
    crate-private absolute targets (open decision).
    Outcome (2026-10-26): A.
  - Finding 3: 408 and 5xx retry stops drop the Retry-After delay. Severity
    Medium, effort Large. Recommended: B, `Unavailable` carries `retry_after`.
    Outcome (2026-10-26): B.
  - Finding 4: Relative proxy target with a backslash can bypass the origin
    check. Severity Medium, effort Small. Recommended: B, reject backslashes
    (open decision).
    Outcome (2026-10-26): C.
  - Finding 5: Managed-auth live test cannot detect proxy failures. Severity
    Medium, effort Small. Recommended: A, Google Calendar targets and outcome
    assertions.
    Outcome (2026-10-26): A.
  - Finding 6: Unit tests miss four resource rules. Severity Medium, effort
    Small. Recommended: A, add the four tests.
    Outcome (2026-10-26): B.
  - Finding 7: Accounts protocol omits the ownership outcomes. Severity Medium,
    effort Small. Recommended: A, record the requirements.
    Outcome (2026-10-26): A.
  - Finding 8: Connection webhooks have no documented signature. Severity
    Medium, effort Small. Recommended: B, an unauthenticated-event requirement.
    Open (2026-10-26): no answer yet.
  - Finding 9: Trigger context timestamp rule can reject deliveries. Severity
    Medium, effort Small. Recommended: B, best-effort `ts` decoding.
    Outcome (2026-10-26): C.
  - Finding 13: Oversize 401 through reqwest keeps the bearer. Severity Low,
    effort Large. Recommended: B, status on `ResponseTooLarge`.
    Outcome (2026-10-26): B.
  - Finding 14: Execution check parses every body twice. Severity Low, effort
    Small. Recommended: A, decode only the `error` member.
    Outcome (2026-10-26): A.
  - Finding 15: Dispatch copies every request body. Severity Low, effort Small.
    Recommended: A, move the body when no retry follows.
    Outcome (2026-10-26): B.
  - Finding 18: No gate builds the client without default features. Severity
    Low, effort Small. Recommended: A, add no-default steps.
    Open (2026-10-26): explanation requested.
  - Finding 21: Component retrieval keeps a hand-written query encoder. Severity
    Low, effort Small. Recommended: A, use `with_query`.
    Outcome (2026-10-26): A.
  - Finding 22: Protocol sentences promise optional inputs and cursors. Severity
    Low, effort Small. Recommended: A, correct the three sentences.
    Open (2026-10-26): question asked about OpenAPI generation.
  - Finding 23: Serde path skips connection event checks. Severity Low, effort
    Small. Recommended: B, one constructor for both paths.
    Outcome (2026-10-26): B.
  - Finding 25: Serde round trip changes delivery identity. Severity Low, effort
    Small. Recommended: A, document it.
    Open (2026-10-26): explanation of trade-offs requested.
  - Finding 26: Webhook status mapping blames the sender for host faults.
    Severity Low, effort Small. Recommended: A, map host faults to 500.
    Outcome (2026-10-26): C.
  - Finding 27: Protocol does not define an invalid context ID. Severity Low,
    effort Small. Recommended: A, record the rule.
    Outcome (2026-10-26): A.
  - Finding 28: Webhook header cases and status mapping lack tests. Severity
    Low, effort Small. Recommended: B, a requirement table test.
    Outcome (2026-10-26): A.
  - Finding 29: A failed CLI result write hides a completed change. Severity
    Low, effort Medium. Recommended: B, per-command text after a sent request.
    Outcome (2026-10-26): B.
  - Finding 31: CLI error hints state a rule the input meets. Severity Low,
    effort Small. Recommended: A, state the whole rule.
    Outcome (2026-10-26): A.
  - Finding 32: A zero rate-limit wait prints "Wait 0 seconds". Severity Low,
    effort Small. Recommended: B, a non-zero wait type.
    Outcome (2026-10-26): B.
  - Finding 33: Integers in number fields re-encode as floats. Severity Low,
    effort Medium. Recommended: B, `serde_json::Number` and integer samples.
    Outcome (2026-10-26): A.
  - Finding 37: Unused public `ProjectWebhook` model. Severity Low, effort
    Medium. Recommended: A, delete it.
    Outcome (2026-10-26): A.
  - Finding 38: Unused chrono clock features. Severity Low, effort Small.
    Recommended: A, disable chrono default features.
    Outcome (2026-10-26): A.
  - Finding 39: Local gate rewrites a stale Cargo.lock. Severity Low, effort
    Small. Recommended: A, `--locked` in the xtask alias.
    Outcome (2026-10-26): A.
  - Finding 40: Gate failure path has no unit test. Severity Low, effort Small.
    Recommended: A, unimock tests.
    Outcome (2026-10-26): A.
  - Finding 41: CI repeats the gate instead of running it. Severity Low, effort
    Small. Recommended: A, run `cargo xtask check` in CI.
    Outcome (2026-10-26): A.
  - Finding 42: Protocol promises lossless values that models round. Severity
    Low, effort Small. Recommended: A, state the number limit.
    Outcome (2026-10-26): A.
  - Finding 48: Plan evidence lines name lost worktree files. Severity Low,
    effort Small. Recommended: A, keep new evidence in the plan folder.
    Outcome (2026-10-26): A.
  - Finding 49: Plan rewrote two checked TODOs. Severity Low, effort Small.
    Recommended: A, restore the words and add correction TODOs.
    Outcome (2026-10-26): A.
  - Finding 50: `Emitter::Other` and unknown-field maps print raw JSON in Debug.
    Severity Low, effort Medium. Recommended: B, mask secret-like names (open
    decision).
    Outcome (2026-10-26): A.
  - Finding 52: `RateLimited` drops `x-pd-rate-limit-reset-in`. Severity Low,
    effort Small. Recommended: parse it as a reset delay.
    Outcome (2026-10-26): A.
  - Finding 53: The vendored OpenAPI snapshot is behind the live schema.
    Severity Low, effort Medium. Recommended: refresh it in a separate change
    and review the diff.
    Outcome (2026-10-26): A.
  - Finding 55: No test checks `InvalidJson` after the webhook parse fix.
    Severity Low, effort Small. Recommended: B, one decoder error table test.
    Outcome (2026-10-26): B.
  - Finding 56: The OAuth coverage row skips the OpenAPI method and path.
    Severity Low, effort Small. Recommended: B, check every row in one loop.
    Outcome (2026-10-26): A.
  - Finding 63: Crate docs carry broken relative links in rustdoc. Severity Low,
    effort Small. Recommended: A, absolute repository URLs after the merge.
    Outcome (2026-10-26): A.
  - Finding 64: `with_headers` merges on `ApiRequest` but replaces on
    `ProxyRequest`. Severity Low, effort Small. Recommended: B, merge on both.
    Outcome (2026-10-26): B.
  - Finding 65: Several commit bodies hold literal `\n` text, and one title is
    too long. Severity Low, effort Small. Recommended: A, keep the history.
    Outcome (2026-10-26): A.
  - Follow-up decision: ban `Result::map_err` outside tests with a clippy
    `disallowed-methods` entry. Effort Small. Covers findings 59 and 60.
    Previous examples: b364bb2, 85d7679, a4c09f1.
    Outcome (2026-10-26): A, add the ban (decision 75).
  - Follow-up decision: an xtask check that fails when `<name>.rs` sits beside
    `<name>/`. Effort Medium. Covers findings 36, 47, and 61.
    Outcome (2026-10-26): A, add the check (decision 76).
  - Outside the scope: `AGENTS.md` intro paragraph differs from the template.
    Separate branch.
  - Outside the scope: `docs/protocol/webhooks.md` says the client preserves
    unknown fields; the decoder does. Separate branch.
  - Outside the scope: the client README never names
    `tests/live_managed_auth.rs`. Separate branch.
  - Outside the scope: `accounts_tests.rs` and `tokens_tests.rs` use inline
    `pipedream_connect_types::` paths. Separate branch.
  - Outside the scope: the library packages do not include the license file.
    Separate branch.
  - The full report is `.context/pipedream-connect-sdk/final-review/report.md`.

## Milestone 13: Apply the final review decisions

Summary: apply the SDK, docs, and workspace outcomes that the user chose on
2026-10-26. Finding and decision numbers refer to the final review report.

- [x] Types crate: reject `.` and `..` identifiers (66 A), cursors take
      visible ASCII up to 2 KiB (70 B), `serde_json::Number` for the four
      `number` fields (33 A), delete `ProjectWebhook` (37 A), and one shared
      date type for every date field (9 C).
- [x] Client core: `retry_after` on `Unavailable` (3 B), the status on
      `ResponseTooLarge` (13 B), the `x-pd-rate-limit-reset-in` delay (52 A),
      decode only the `error` member (14 A), shared `Bytes` bodies (15 B),
      merging `ProxyRequest::with_headers` (64 B), GET retry defaults (67 A),
      every execution 202 is `Accepted` (73 B), `with_query` for component
      retrieval (21 A), and backslash and raw `@` proxy target rejection
      through the shared URL rules (4 C).
- [x] Webhooks crate: one connection constructor for both paths (23 B), a
      fault class on each error (26 C), header cases (28 A), one decoder
      error table (55 B), and a 10 MiB default body limit (68 B).
- [x] Tests: managed-auth live test targets and assertions (5 A), a shared
      `assert_wire` helper with the four missing rules (6 B), and OpenAPI
      values for the OAuth coverage row (56 A).
- [x] Docs: the workflow bearer risk (1 A), the account ownership outcomes
      (7 A), the invalid context ID rule (27 A), the number limit (42 A), and
      every protocol sentence that the items above change.
- [x] Workspace: chrono without default features (38 A), `--locked` in the
      xtask alias (39 A), gate failure tests (40 A), CI runs
      `cargo xtask check` (41 A), a clippy ban on `map_err` (75 A), and an
      xtask check for `<name>.rs` beside `<name>/` (76 A).
- [x] Plan: restore the words of the two rewritten checked TODOs and add
      correction TODOs (49 A).
- [x] Run the gate, commit, and push.

## Milestone 14: CLI updates for the review decisions

Tags: ui

Summary: apply the CLI outcomes and adapt the CLI to Milestone 13.

- [x] One per-command text table for every failure after a sent request
      (29 B).
- [x] Hints that state the whole rule (31 A).
- [x] A non-zero rate-limit wait type (32 B).
- [x] Adapt the CLI to the Milestone 13 error, identifier, cursor, retry, and
      202 changes, with tests and README updates.
- [x] Run the gate and the credential-free smoke, commit, and push.

## Milestone 15: Verification, commit, and review of the decisions round

Summary: the final checks and one review loop for Milestones 13 and 14.

- [x] Run the targeted tests and the complete gate named in `AGENTS.md`.
- [x] Commit with Conventional Commits and push the branch.
- [x] Review the diff since `d748ed5` with the `implementation-review` skill
      after the push. Apply the review-fix rule: fix the `Auto-fix: yes`
      findings, re-review once, and report the rest.
      Evidence: `.context/review-r2/sdk-summary.md`,
      `.context/review-r2/cli-workspace-summary.md`, and
      `.context/review-r2/rereview/report.md`.
  - Review: findings S1 to S8 for the SDK and R1 to R7 for the CLI and
    workspace. Every finding except S2 is fixed in 62024a1, eee82bc, 4fa6afa,
    46b96b5, 95fc0e8, and 1673a28.
  - Re-review: 11 of 14 fixes verified. S1, S3, and R4 were partial, and the
    re-review found 4 Low findings. The partial parts and the 4 findings are
    fixed in 3f11122 and 6593d24.
  - S2: The shared date type adds new decode rejections. Severity Low, effort
    Small. Recommended: A, parse date strings with chrono's
    `DateTime<FixedOffset>` parser.
    Open (2026-10-26): needs a user decision.
  - Follow-up: add `#![warn(unreachable_pub)]` to the xtask binary and narrow
    each item that it flags. Effort Small. Covers re-review finding 4.
    Previous examples: none found.
  - Outside the scope: `xtask/src/layout.rs` and `xtask/src/file_length.rs`: a
    read error names the workspace root, not the directory that failed.
    Separate branch.
  - Outside the scope: `xtask/src/command.rs`: `CommandSpec`, `CommandOutcome`,
    and `CommandRunner` are `pub` in a binary crate. Separate branch.
  - Outside the scope: `crates/pipedream-connect-types/src/triggers/emitters.rs`:
    `TimerInterface.schedule_changed_at` reads integers as epoch milliseconds,
    but the OpenAPI document gives it no unit. Separate branch.
  - Outside the scope: `crates/pipedream-connect/src/failure.rs`: the
    `provider_message` doc comment describes history. Separate branch.
  - Outside the scope: `xtask/src/error.rs`: a failed command prints its exit
    code as `Some(101)`. Separate branch.
  - Outside the scope: `crates/pipedream-cli`: a request to a closed port
    reports that the change "may have run". Separate branch.
- [ ] Open the pull request when every other milestone TODO is ticked, or when
      the user asks for it.

## Post-merge follow-up (non-blocking)

- [ ] Open a plan in `futex-ai/firna` that makes `fna-connectors-pipedream`
      depend on the SDK and keeps its `ConnectorProvider` mapping.
- [ ] Publish the library crates to crates.io when `internal-error` is
      published.
- [ ] Consider an MCP server client and the legacy workflow REST API as
      separate plans.
- [ ] Use absolute repository URLs in the Related Docs lists of the library
      READMEs, so that rustdoc links work (finding 63, outcome A).
- [ ] Refresh the vendored OpenAPI fixture in a separate change, implement
      any new operation, and review the diff (finding 53, outcome A).
