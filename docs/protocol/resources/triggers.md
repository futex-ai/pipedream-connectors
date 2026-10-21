# Trigger operations

These operations discover registry triggers and manage deployed triggers for
an external user.

## Operation inventory

<!-- markdownlint-disable MD013 -->

| Method | Path | OpenAPI operationId |
| --- | --- | --- |
| GET | `/v1/connect/{project_id}/deployed-triggers` | `listDeployedTriggers` |
| GET | `/v1/connect/{project_id}/deployed-triggers/{trigger_id}` | `retrieveDeployedTrigger` |
| PUT | `/v1/connect/{project_id}/deployed-triggers/{trigger_id}` | `updateDeployedTrigger` |
| DELETE | `/v1/connect/{project_id}/deployed-triggers/{trigger_id}` | `deleteDeployedTrigger` |
| GET | `/v1/connect/{project_id}/deployed-triggers/{trigger_id}/events` | `listDeployedTriggerEvents` |
| GET | `/v1/connect/{project_id}/deployed-triggers/{trigger_id}/pipelines` | `listDeployedTriggerWorkflows` |
| PUT | `/v1/connect/{project_id}/deployed-triggers/{trigger_id}/pipelines` | `updateDeployedTriggerWorkflows` |
| GET | `/v1/connect/{project_id}/deployed-triggers/{trigger_id}/webhooks` | `listDeployedTriggerWebhooks` |
| PUT | `/v1/connect/{project_id}/deployed-triggers/{trigger_id}/webhooks` | `updateDeployedTriggerWebhooks` |
| GET | `/v1/connect/{project_id}/deployed-triggers/{trigger_id}/webhooks/{webhook_id}` | `retrieveDeployedTriggerWebhook` |
| POST | `/v1/connect/{project_id}/deployed-triggers/{trigger_id}/webhooks/{webhook_id}/regenerate_signing_key` | `regenerateDeployedTriggerWebhookSigningKey` |

<!-- markdownlint-enable MD013 -->

## Deployment lifecycle

A deployment associates a registry trigger with an external user and its
configured properties.

A deployment MAY specify a webhook URL, a workflow endpoint, and whether to
emit an event during deployment.

A deployment returns the deployed trigger identity and its emitter metadata.

A deployed trigger list uses cursor pagination and MAY filter by emitter type.

A deployed trigger retrieve operation returns the selected deployment.

An update operation changes only the fields included by the request.

A delete operation MAY ignore webhook delivery errors when requested.

A delete operation for an already missing deployment returns success when the
request enables missing-resource tolerance.

Every lifecycle operation MUST include both the project and deployed trigger
identity.

## Event history and workflows

Event history returns recent events for one deployed trigger.

The event history limit is between 1 and 100 inclusive.

Workflow list and update operations manage the workflow endpoints associated
with one deployment.

## Webhook destinations

Webhook list and update operations manage destinations for one deployment.

A webhook retrieve operation returns the destination and signing-key metadata.

A signing-key regeneration operation replaces the selected destination key.

The client MUST preserve returned signing keys as credential-class values.

The client MUST use the trigger-level signing key when the delivered webhook
came from a trigger destination.

The project environment signing key is the fallback when no trigger-level key
is configured.
