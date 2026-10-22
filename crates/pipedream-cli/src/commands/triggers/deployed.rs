//! Deployed trigger commands: `triggers list`, `get`, `delete`, and `events`.

use std::sync::Arc;

use pipedream_connect::pagination::FilteredPageSource;
use pipedream_connect::resources::deployed_triggers::DeployedTriggersApi;
use pipedream_connect_types::{
    DeleteTriggerRequest, DeployedTriggerId, DeployedTriggerQuery, ExternalUserId,
    ListDeployedTriggersRequest, ListTriggerEventsRequest,
};

use crate::argument::Argument;
use crate::commands::numbers::bounded;
use crate::commands::pages::{self, Paging};
use crate::commands::triggers::args::{
    DeleteTriggerArgs, ListTriggersArgs, TriggerArgs, TriggerEventsArgs,
};
use crate::document::{Acknowledgement, Document, Output, Secrets};
use crate::error::{Result, called, checked};
use crate::target::Target;

/// The event count that Pipedream returns when a request omits it.
const DEFAULT_EVENTS: u16 = 20;

/// A validated command on deployed triggers.
#[derive(Debug, PartialEq)]
pub(crate) enum DeployedTask {
    /// List the deployed triggers of one external user.
    List {
        /// The filters and the first page.
        request: ListDeployedTriggersRequest,
        /// The page budget.
        paging: Paging,
    },
    /// Read one deployed trigger.
    Get(DeployedTriggerId, DeployedTriggerQuery),
    /// Delete one deployed trigger.
    Delete(DeployedTriggerId, DeleteTriggerRequest),
    /// List the recent events of one deployed trigger.
    Events(DeployedTriggerId, ListTriggerEventsRequest),
}

/// Validates `triggers list`. Sends no request.
pub(super) fn list(args: ListTriggersArgs) -> Result<DeployedTask> {
    let paging = args.pages.paging()?;
    let request = ListDeployedTriggersRequest {
        external_user_id: external_user(args.external_user_id)?,
        page: paging.page.clone(),
        emitter_type: args.emitter_type.map(|kind| kind.model()),
    };
    Ok(DeployedTask::List { request, paging })
}

/// Validates `triggers get`. Sends no request.
pub(super) fn get(args: TriggerArgs) -> Result<DeployedTask> {
    let (trigger, external_user_id) = trigger(args)?;
    Ok(DeployedTask::Get(
        trigger,
        DeployedTriggerQuery { external_user_id },
    ))
}

/// Validates `triggers delete`. Sends no request.
pub(super) fn delete(args: DeleteTriggerArgs) -> Result<DeployedTask> {
    let (trigger, external_user_id) = trigger(args.trigger)?;
    let request = DeleteTriggerRequest {
        external_user_id,
        ignore_hook_errors: args.ignore_hook_errors.then_some(true),
        missing_ok: args.missing_ok,
    };
    Ok(DeployedTask::Delete(trigger, request))
}

/// Validates `triggers events`. Sends no request.
pub(super) fn events(args: TriggerEventsArgs) -> Result<DeployedTask> {
    let (trigger, external_user_id) = trigger(args.trigger)?;
    let count = match args.limit {
        Some(limit) => bounded(&limit),
        None => DEFAULT_EVENTS,
    };
    let request = checked(
        ListTriggerEventsRequest::new(external_user_id, count),
        Argument::Limit,
    )?;
    Ok(DeployedTask::Events(trigger, request))
}

fn trigger(args: TriggerArgs) -> Result<(DeployedTriggerId, ExternalUserId)> {
    Ok((
        checked(DeployedTriggerId::new(args.trigger), Argument::Trigger)?,
        external_user(args.external_user_id)?,
    ))
}

fn external_user(value: String) -> Result<ExternalUserId> {
    checked(ExternalUserId::new(value), Argument::ExternalUserId)
}

/// Runs a validated command against the deployed trigger trait object.
pub(crate) async fn run(
    task: DeployedTask,
    api: Arc<dyn DeployedTriggersApi + Send + Sync>,
    secrets: Secrets,
) -> Result<Output> {
    match task {
        DeployedTask::List { request, paging } => {
            let source = FilteredPageSource::new(request, move |request| {
                let api = api.clone();
                async move { api.list(request).await }
            });
            pages::list(Arc::new(source), paging, Target::Triggers, secrets).await
        }
        DeployedTask::Get(trigger, query) => {
            let response = called(api.get(trigger, query).await, Target::Trigger)?;
            Ok(Document::redacted(response, secrets)?.into())
        }
        DeployedTask::Delete(trigger, request) => {
            called(api.delete(trigger, request).await, Target::TriggerDelete)?;
            Ok(Document::redacted(Acknowledgement::deleted(), secrets)?.into())
        }
        DeployedTask::Events(trigger, request) => {
            let response = called(
                api.list_events(trigger, request).await,
                Target::TriggerEvents,
            )?;
            Ok(Document::redacted(response, secrets)?.into())
        }
    }
}

#[cfg(test)]
#[path = "_tests_/deployed/mod.rs"]
mod deployed_tests;
