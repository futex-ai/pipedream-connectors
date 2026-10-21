# Catalog and execution operations

These operations discover app metadata and registry components. They also
configure and execute actions and registry triggers.

## Operation inventory

<!-- markdownlint-disable MD013 -->

| Method | Path | OpenAPI operationId |
| --- | --- | --- |
| GET | `/v1/connect/app_categories` | `listAppCategories` |
| GET | `/v1/connect/app_categories/{id}` | `retrieveAppCategory` |
| GET | `/v1/connect/apps` | `listApps` |
| GET | `/v1/connect/apps/{app_id}` | `retrieveApp` |
| GET | `/v1/connect/{project_id}/actions` | `listActions` |
| GET | `/v1/connect/{project_id}/actions/{component_id}` | `retrieveAction` |
| POST | `/v1/connect/{project_id}/actions/configure` | `configureActionProp` |
| POST | `/v1/connect/{project_id}/actions/props` | `reloadActionProps` |
| POST | `/v1/connect/{project_id}/actions/run` | `runAction` |
| GET | `/v1/connect/{project_id}/components` | `listComponents` |
| GET | `/v1/connect/{project_id}/components/{component_id}` | `retrieveComponent` |
| POST | `/v1/connect/{project_id}/components/configure` | `configureComponentProp` |
| POST | `/v1/connect/{project_id}/components/props` | `reloadComponentProps` |
| GET | `/v1/connect/{project_id}/triggers` | `listTriggers` |
| GET | `/v1/connect/{project_id}/triggers/{component_id}` | `retrieveTrigger` |
| POST | `/v1/connect/{project_id}/triggers/configure` | `configureTriggerProp` |
| POST | `/v1/connect/{project_id}/triggers/deploy` | `deployTrigger` |
| POST | `/v1/connect/{project_id}/triggers/props` | `reloadTriggerProps` |

<!-- markdownlint-enable MD013 -->

## Catalog queries

App, component, action, and trigger list operations support cursor pagination.

The category list returns a bare array and does not accept query parameters.

App lists MAY filter by query, category, sort key, sort direction, and whether
components, actions, or triggers exist.

Component, action, and trigger lists MAY filter by query, app, registry, and
component type when the operation defines that filter.

A retrieve operation MUST return the requested resource identity or an error.

A response that has a different identity from the request MUST be rejected as
an internal identity error.

## Property configuration

A configure operation accepts the component identity, external user identity,
property name, and the current configuration context.

A props operation accepts the component identity, external user identity, and
configured properties.

A configure or props operation MAY return updated options, observations,
context, dynamic properties, and provider errors.

A configure or props operation MAY return 202 when the request is non-blocking.

## Action execution

An action run accepts an action identity, external user identity, configured
properties, and an optional stash reference.

An action run does not accept a blocking flag.

An action run that returns 202 returns an accepted asynchronous result without
decoding a result body.

A structured execution error preserves the provider attribution, name, and
message fields described by the client error contract.
