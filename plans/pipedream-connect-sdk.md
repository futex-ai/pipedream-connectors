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

## Open decisions

- **GET retry defaults (2026-10-21).** Decision 4 requires an opt-in GET
  retry policy but does not settle the default total attempts or maximum
  delay. **A.** Three total attempts, with a 30-second maximum delay and
  `Retry-After` support. Effort: Medium. **B.** Two total attempts, with a
  60-second maximum delay and `Retry-After` support. Effort: Medium.
  **Recommendation:** A, because it matches the settled two-retry wording
  while bounding delay and work. The protocol leaves this default open.
- **Webhook body limit (2026-10-21).** Milestone 9 requires a bounded webhook
  body but does not settle its default. **A.** Use a 1 MiB default, with a
  configurable limit. Effort: Small. **B.** Use the 10 MiB client response
  default, with a configurable limit. Effort: Small. **Recommendation:** A,
  because webhook ingress should have a smaller independent bound. The
  protocol leaves this default open.

- **Literal dot path parameters (2026-10-21).** The URL library skips `.`
  and `..` when appending path segments. Some generic identifiers currently
  accept those values. **A.** Reject literal dot identifiers before a token
  call and align the types grammar. Effort: Medium. **B.** Preserve literal
  dot parameters through an HTTP URL representation that does not normalize
  them. Effort: Large. **Recommendation:** A, because it prevents a route
  change with one identifier rule. No new rejection is applied before the
  user answers. This choice blocks final handling of these identifiers in
  resource paths.

- **Category list shape (2026-10-21).** The saved OpenAPI schema and the
  current public category-list page return a bare array without query
  parameters. The milestone guidance asks every list to take a typed request
  and return `Page<T>`. **A.** Return `Vec<AppCategory>` from `list()` and
  document this non-paged exception. Effort: Small. **B.** Take an empty typed
  request and normalize the array into `Page<AppCategory>` with absent page
  metadata. Effort: Medium. **Recommendation:** A, because it matches the
  provider contract without fabricated page data. No list option is applied
  before the user answers. This blocks category listing and its tests.

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
pipedream-cli triggers deploy --external-user-id user_1 \
  --component google_calendar-new-or-updated-event-instant \
  --props props.json --webhook-url https://example.com/hook
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
- [ ] Update the READMEs and the resources protocol doc only if a contract
      changed.
- [ ] Run the gate, commit, and push.

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
- [x] Update the READMEs and the resources protocol doc only if a contract
      changed.
- [x] Run the complete gate and commit the branch without pushing.
- [ ] Run the trigger live test against a development project. Blocked: the
      required Pipedream credential and connected-account variables are absent.

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

- [ ] Create `crates/pipedream-cli` with `clap`. Mark it `publish = false`.
      Give it a `README.md` in the required section order.
- [ ] Shared configuration flags, the `PIPEDREAM_*` variables, JSON output,
      and exit codes.
- [ ] Registry and execution commands: `apps list`, `apps get`,
      `categories list`, `components list`, `components get`,
      `actions list`, and `actions run`.
- [ ] Managed-auth and proxy commands: `tokens create`, `accounts list`,
      `accounts get`, `users list`, and `proxy <method>`. `tokens create`
      prints the Connect Link URL for an external user.
- [ ] Trigger commands: `triggers list`, `triggers get`, `triggers deploy`,
      `triggers delete`, and `triggers events`.
- [ ] Administration commands: `project info`, `usage`, and `webhook get`.
- [ ] CLI unit tests with mocked resource traits for every command:
      argument parsing, request mapping, JSON output, and exit codes.
- [ ] The CLI README with the command reference. Add the CLI to the
      workspace README, and the CLI smoke command to `AGENTS.md`.
- [ ] Run the gate, run every command against a development project,
      commit, and push.

## Milestone 11: Smoke and release readiness

Summary: the SDK and the CLI are usable by a stranger.

- [ ] Each library crate `lib.rs` includes its README as crate docs, so the
      Quick Start compiles as a doctest.
- [ ] `cargo doc --no-deps` builds without warnings, and `Cargo.toml`
      carries description, repository, keywords, and categories.
- [ ] Run the complete smoke against a development project: every CLI
      command and both live tests. Save the output under
      `.context/pipedream-connect-sdk/smoke.md`.
- [ ] Update the workspace README and the architecture doc for anything
      learnt.
- [ ] Run the gate, commit, and push.

## Milestone 12: Verification, commit, and review

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
- [ ] Publish the library crates to crates.io when `internal-error` is
      published.
- [ ] Consider an MCP server client and the legacy workflow REST API as
      separate plans.
