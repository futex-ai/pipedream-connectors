# Accounts and users operations

These operations manage connected accounts in a project and partition them by
external user.

## Operation inventory

<!-- markdownlint-disable MD013 -->

| Method | Path | OpenAPI operationId |
| --- | --- | --- |
| GET | `/v1/connect/{project_id}/accounts` | `listAccounts` |
| POST | `/v1/connect/{project_id}/accounts` | `createAccount` |
| GET | `/v1/connect/{project_id}/accounts/{account_id}` | `retrieveAccount` |
| DELETE | `/v1/connect/{project_id}/accounts/{account_id}` | `deleteAccount` |
| DELETE | `/v1/connect/{project_id}/apps/{app_id}/accounts` | `deleteAccountByApp` |
| GET | `/v1/connect/{project_id}/users` | `listUsers` |
| DELETE | `/v1/connect/{project_id}/users/{external_user_id}` | `deleteUserExternalUser` |
| GET | `/v1/connect/{project_id}/users/{external_user_id}/accounts` | `listUserAccounts` |

<!-- markdownlint-enable MD013 -->

## Account queries and credentials

Account list operations support external user, app, OAuth app, cursor, and
`include_credentials` filters when the operation defines them.

An account retrieve operation supports `include_credentials`.

`include_credentials=false` is the default.

`include_credentials=true` MAY return the account's credential object for
bring-your-own-OAuth apps.

The client MUST wrap every returned credential value as a credential-class
value before exposing diagnostics.

The client MUST preserve app-specific credential fields inside the credential
object.

Account and user list operations use the common cursor contract.

## Account creation and deletion

Account creation accepts an external user, app or OAuth app, configuration
mapping, and an optional Connect token as defined by the operation.

Deleting an account removes only the selected account.

Deleting accounts by app removes the selected app's accounts in the project
scope.

Deleting an external user removes the user and all resources that Pipedream
associates with that user.

The client MUST send a project and external user value on every user-scoped
operation.
