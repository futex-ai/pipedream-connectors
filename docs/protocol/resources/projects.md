# Projects and administration operations

These operations manage project metadata, app overrides, OAuth clients,
project webhooks, usage, and file stash downloads.

## Operation inventory

<!-- markdownlint-disable MD013 -->

| Method | Path | OpenAPI operationId |
| --- | --- | --- |
| GET | `/v1/connect/{project_id}/app_overrides` | `listAppOverrides` |
| POST | `/v1/connect/{project_id}/app_overrides` | `createAppOverride` |
| GET | `/v1/connect/{project_id}/app_overrides/{id}` | `retrieveAppOverride` |
| PUT | `/v1/connect/{project_id}/app_overrides/{id}` | `updateAppOverride` |
| DELETE | `/v1/connect/{project_id}/app_overrides/{id}` | `deleteAppOverride` |
| GET | `/v1/connect/{project_id}/file_stash/download` | `downloadFileStashFile` |
| GET | `/v1/connect/{project_id}/oauth_apps` | `listOauthApps` |
| POST | `/v1/connect/{project_id}/oauth_apps` | `createOauthApp` |
| GET | `/v1/connect/{project_id}/oauth_apps/{id}` | `retrieveOauthApp` |
| PUT | `/v1/connect/{project_id}/oauth_apps/{id}` | `updateOauthApp` |
| DELETE | `/v1/connect/{project_id}/oauth_apps/{id}` | `deleteOauthApp` |
| GET | `/v1/connect/projects` | `listProjects` |
| POST | `/v1/connect/projects` | `createProject` |
| GET | `/v1/connect/projects/{project_id}` | `retrieveProject` |
| DELETE | `/v1/connect/projects/{project_id}` | `deleteProject` |
| PATCH | `/v1/connect/projects/{project_id}` | `updateProject` |
| POST | `/v1/connect/projects/{project_id}/logo` | `updateProjectLogo` |
| GET | `/v1/connect/{project_id}/projects/info` | `retrieveProjectInfo` |
| GET | `/v1/connect/{project_id}/webhook` | `retrieveProjectEnvironmentWebhook` |
| PUT | `/v1/connect/{project_id}/webhook` | `updateProjectEnvironmentWebhook` |
| DELETE | `/v1/connect/{project_id}/webhook` | `deleteProjectEnvironmentWebhook` |
| POST | `/v1/connect/{project_id}/webhook/regenerate_signing_key` | `regenerateProjectEnvironmentWebhookSigningKey` |

<!-- markdownlint-enable MD013 -->

## Project and app administration

Project list and retrieve operations return project metadata in the account
scope allowed by the bearer.

Project update operations change only fields included by the request.

Project logo updates accept the operation's binary or encoded logo body.

Project information returns apps enabled for the selected project environment.

App override list operations support cursor pagination.

OAuth app list operations support cursor pagination.

The client MUST preserve OAuth client secrets as credential-class values.

## Project webhooks

A project environment webhook has one URL and one signing key.

The project environment is selected by `x-pd-environment`.

Updating or regenerating a project webhook returns the new signing key when
Pipedream provides it.

Deleting a project webhook removes the default destination for that
environment.

The client MUST redact the signing key in diagnostics.

## File stash

A file stash download returns bytes or a redirect location.

The client MUST NOT follow a file stash redirect.
