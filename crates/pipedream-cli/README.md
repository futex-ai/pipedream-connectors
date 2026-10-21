# pipedream-cli

`pipedream-cli` is the command line of the Pipedream Connect SDK. It runs SDK
operations against a Pipedream project and prints each result as JSON. Use it
for smoke tests and to inspect Connect data. The crate is not published.
Library users depend on `pipedream-connect` and `pipedream-connect-types`.

## Responsibilities

- Read connection settings from flags or `PIPEDREAM_*` variables.
- Validate every argument and input file before any request.
- Run one Connect operation for each command through the resource traits.
- Print one JSON document for each successful command, with credentials
  redacted.
- Report each failure with one message and a documented exit code.

## What This Crate Does

The CLI is a composition root. It builds one `PipedreamClient` for each run
with the reqwest transport, the system clock, and the Tokio sleeper. Each
command receives only the resource trait object that it needs. Command code
never names the concrete client. The client never retries a mutation, and the
CLI adds no retries.

### Configuration

Each connection setting has a flag and a variable. The flag takes precedence.
An empty variable counts as unset. The global flags work before or after the
command.

<!-- markdownlint-disable MD013 -->

| Flag | Variable | Required | Value |
| --- | --- | --- | --- |
| `--client-id` | `PIPEDREAM_CLIENT_ID` | Yes | OAuth client ID |
| `--client-secret` | `PIPEDREAM_CLIENT_SECRET` | Yes | OAuth client secret |
| `--project-id` | `PIPEDREAM_PROJECT_ID` | Yes | Project ID, `proj_...` |
| `--environment` | `PIPEDREAM_ENVIRONMENT` | Yes | `development` or `production` |
| `--api-origin` | `PIPEDREAM_API_ORIGIN` | No | HTTPS origin |

<!-- markdownlint-enable MD013 -->

Set the client secret with the variable. Shell history and process lists can
show flag values. Help text never shows a configured value. A flag value that
starts with a hyphen needs the `=` form, such as `--client-secret=-abc`. The
environment has no default, because development and production use separate
credentials.

The API origin defaults to `https://api.pipedream.com`. When a flag or a
variable selects another origin, the CLI writes one note to standard error
before any request. The note names the origin and the flag or variable that
set it.

The CLI reports every missing required setting in one message.

### Commands

The CLI implements the commands whose SDK resource traits exist. The other
commands are planned. They wait for the milestone that adds their resource.

<!-- markdownlint-disable MD013 -->

| Command | Status | What it does |
| --- | --- | --- |
| `apps list` | Available | Lists catalog apps. |
| `apps get <APP>` | Available | Reads one app by slug or ID. |
| `categories list` | Available | Lists every app category. |
| `components list` | Available | Lists registry components. |
| `components get <COMPONENT>` | Available | Reads one component and its props. |
| `actions list` | Available | Lists registry actions. |
| `actions run <ACTION>` | Available | Runs one action for an external user. |
| `tokens create` | Planned, Milestone 6 | Creates a Connect token for an external user. |
| `accounts list` | Planned, Milestone 6 | Lists connected accounts. |
| `accounts get` | Planned, Milestone 6 | Reads one connected account. |
| `users list` | Planned, Milestone 6 | Lists external users. |
| `proxy get`, `post`, `put`, `patch`, `delete` | Planned, Milestone 6 | Sends one request through the Connect proxy. |
| `project info` | Planned, Milestone 6 | Lists the apps enabled in the project environment. |
| `triggers list` | Planned, Milestone 7 | Lists the deployed triggers of an external user. |
| `triggers get` | Planned, Milestone 7 | Reads one deployed trigger. |
| `triggers deploy` | Planned, Milestone 7 | Deploys a registry trigger. |
| `triggers delete` | Planned, Milestone 7 | Deletes a deployed trigger. |
| `triggers events` | Planned, Milestone 7 | Lists recent events of a deployed trigger. |
| `usage` | Planned, Milestone 8 | Lists Connect usage for a time window. |
| `webhook get` | Planned, Milestone 8 | Reads the project environment webhook. |

<!-- markdownlint-enable MD013 -->

`apps list` takes these filters:

- `--query <TEXT>` searches the catalog.
- `--sort-key <name|name_slug|featured_weight>` and
  `--sort-direction <asc|desc>` sort the apps.
- `--category <ID>` keeps apps in a category. Repeat it for more categories.
- `--has-components`, `--has-actions`, and `--has-triggers` keep apps that
  have them. Add `false` to keep apps without them.

`components list` and `actions list` take `--query <TEXT>`, `--app <APP>`,
and `--registry <public|private|all>`. `components list` also takes
`--type <action|trigger>`. `components get` takes `--version <VERSION>`.
Without it, Pipedream returns the latest version.

`actions run <ACTION>` takes these arguments:

- `--external-user-id <ID>` names the external user. It is required.
- `--props <FILE>` reads the configured props as a JSON object. Pass `-` to
  read standard input.
- `--version <VERSION>` selects an action version.
- `--dynamic-props-id <ID>` passes the ID from an earlier props reload.
- `--stash-id <STASH>` selects a File Stash. `NEW`, `true`, and `false` are
  selectors. Any other value is a stash ID.

The planned commands follow the same rules. `tokens create` will print the
Connect token, its expiry, and the Connect Link URL. `proxy <method>` will
take `--external-user-id`, `--account`, the target URL, a repeatable
`--header 'Name: value'`, and `--body <FILE>` or `--body -`. It will print the
upstream status, every header, and the body. A UTF-8 body prints as text, and
any other body prints as base64 with its encoding named.

### Pages

Every paged list command takes the same flags:

- `--limit <COUNT>` sets the records for each page, from 1 to 200. The
  default is 100. Any other value fails before a request.
- `--after <CURSOR>` or `--before <CURSOR>` continues from a cursor in the
  `page_info` of an earlier result.
- `--all` reads every page up to the `--max-pages` budget. The default budget
  is 10 pages. The budget has no upper limit.

Without `--all`, the command prints the page as the SDK models it: `data`,
`page_info`, and the fields that the SDK does not model. The SDK model writes
`null` for an absent field that the API documents as nullable. With `--all`,
the command prints one document with the records of every page. It drops the
`page_info` of each page:

```json
{
  "data": [],
  "pages": 10,
  "end": "page_limit",
  "next": { "limit": 100, "after": "cursor" }
}
```

`end` is `exhausted`, `page_limit`, or `stuck_cursor`. At the page limit,
`next` holds the request that continues the listing in the same direction. A
note on standard error names the `--after` or `--before` flag and the quoted
cursor to use. When Pipedream repeats a cursor, the CLI stops, and the result
can contain repeated records. When a page fails, the command prints no
records. For a failure that a retry can fix, the error names the flag that
continues from the failed page.
`categories list` has no page flags, because Pipedream returns every category
in one response.

### Output and redaction

A successful command writes one pretty-printed JSON document and a newline to
standard output. Notes and errors go to standard error. The CLI encodes the
complete document before it writes, so a failed command prints nothing on
standard output. Pipedream fields that the SDK does not model stay in the
output.

The SDK types some values as credentials: account credentials and webhook
signing keys. Each of them prints as `"[REDACTED]"`. Pass `--show-secrets` to
print them. Only a command that exists to produce a credential prints it
without the flag. The planned `tokens create` prints the Connect token and the
Connect Link URL. Provider fields that the SDK does not model print as they
are.

An action run that Pipedream accepts for later completion prints
`{"status": "accepted"}` and exits with code 0.

### Exit codes

| Code | Meaning |
| --- | --- |
| 0 | The command printed its result. |
| 1 | The CLI could not complete the command or read the response. |
| 2 | A flag, a variable, or an input is not valid. No request was sent. |
| 3 | Pipedream rejected the credentials or denied access. |
| 4 | Pipedream did not find the resource. |
| 5 | Pipedream rate limited the request. |
| 6 | Pipedream was unavailable or did not respond. The request may have run. |
| 7 | Pipedream rejected the request, or the action failed. |

### Errors

Each error names what failed and gives one recovery action. Validation errors
name the flag, the variable, or the argument. A JSON input error gives only
the line and the column. JSON that nests more than 127 levels of arrays and
objects gets its own message. Errors never print a secret, a bearer, a
response body, or the provider message. A failed action names where it
failed, for example the component code or the app API with its status. A
network failure names the API origin. After a command that changes data, an
unanswered or unreadable response can mean that the change happened, and the
error says so. A failure that the user cannot fix links to the
[issue tracker](https://github.com/futex-ai/pipedream-connectors/issues).

## Quick Start

Set the connection variables, then run a read command:

```sh
export PIPEDREAM_CLIENT_ID=...
export PIPEDREAM_CLIENT_SECRET=...
export PIPEDREAM_PROJECT_ID=proj_...
export PIPEDREAM_ENVIRONMENT=development
cargo run -p pipedream-cli -- apps list --query calendar --limit 5
cargo run -p pipedream-cli -- components list --app google_calendar --type action
cargo run -p pipedream-cli -- apps list --has-actions --all --max-pages 3
```

Run an action with configured props from a file or from standard input:

```sh
cat > props.json <<'JSON'
{"googleCalendar": {"authProvisionId": "apn_abc123"}, "calendarId": "primary"}
JSON
cargo run -p pipedream-cli -- actions run google_calendar-list-events \
  --external-user-id user_1 --props props.json
echo '{"calendarId": "primary"}' | cargo run -p pipedream-cli -- actions run \
  google_calendar-list-events --external-user-id user_1 --props -
```

Run `cargo run -p pipedream-cli -- --help` or add `--help` to any command for
its flags.

## Development

Run the unit tests and the credential-free smoke test:

```sh
cargo test -p pipedream-cli
cargo test -p pipedream-cli --test cli_smoke
cargo xtask check
```

Unit tests mock the environment variables, the input files, the client
construction, and every resource trait with `unimock`. They capture standard
output and standard error in memory. No unit test touches the network, the
clock, the disk, or the process environment. The smoke test runs the built
binary with an empty environment. It checks the help of every command, the
missing-setting error, and the exit codes without a request to Pipedream.

To add a command, add its group module under `src/commands/`. Define its
`clap` arguments, a `prepare` function that validates them, and a `run`
function that takes one resource trait object. Add a `Target` for its failure
text, an arm in `src/report/target.rs`, and a `Redact` implementation for
each model that it prints.

### Key Code

- `src/main.rs` wires the process boundaries and runs the application.
- `src/app.rs` runs the pipeline: parse, resolve settings, validate input,
  connect once, run, and report.
- `src/cli.rs` defines the global flags and the command tree.
- `src/settings.rs` resolves flags and `PIPEDREAM_*` variables.
- `src/commands/task.rs` validates a command and hands it its trait object.
- `src/commands/pages.rs` defines the page flags and the bounded `--all`
  listing.
- `src/redact.rs` replaces credential-class values before output.
- `src/report/` maps each failure to its exit code and message.
- `tests/cli_smoke.rs` runs the binary without credentials.

### Related Docs

- [SDK architecture](../../docs/architecture.md)
- [Connect client protocol](../../docs/protocol/connect-client.md)
- [Connect resources protocol](../../docs/protocol/connect-resources.md)
- [Client crate README](../pipedream-connect/README.md)
- [SDK plan](../../plans/pipedream-connect-sdk.md)
