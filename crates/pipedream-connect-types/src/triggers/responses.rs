//! Trigger deployment models and requests.

use crate::{Data, EmittedEvent, Emitter, Page, TriggerWebhooks, TriggerWorkflows};

/// Trigger deployment response.
pub type DeployTriggerResponse = Data<Emitter>;

/// Deployed trigger response.
pub type GetTriggerResponse = Data<Emitter>;

/// Deployed trigger list response.
pub type GetTriggersResponse = Page<Emitter>;

/// Deployed trigger event response.
pub type GetTriggerEventsResponse = Data<Vec<EmittedEvent>>;

/// Deployed trigger webhook response.
pub type GetTriggerWebhooksResponse = TriggerWebhooks;

/// Deployed trigger workflow response.
pub type GetTriggerWorkflowsResponse = TriggerWorkflows;
