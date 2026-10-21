//! Registry action commands: `actions list` and `actions run`.

use std::path::PathBuf;
use std::sync::Arc;

use clap::{Args, Subcommand};
use pipedream_connect::pagination::FilteredPageSource;
use pipedream_connect::resources::actions::ActionsApi;
use pipedream_connect_types::{
    ComponentKey, ExternalUserId, ListRegistryRequest, RunActionRequest, StashReference,
};
use serde::Serialize;

use crate::argument::Argument;
use crate::commands::pages::{self, PageArgs, Paging};
use crate::commands::registry::RegistryArgs;
use crate::document::{Document, Output, Secrets};
use crate::error::{Error, Result, checked};
use crate::inputs::{InputSource, Inputs, read_props};
use crate::redact::Redact;
use crate::target::Target;

/// The registry action commands.
#[derive(Debug, Subcommand)]
pub(crate) enum ActionsCommand {
    /// List registry actions.
    List(ListActionsArgs),
    /// Run one action for an external user. The CLI sends the run once and never retries it.
    Run(RunActionArgs),
}

/// The arguments of `actions list`.
#[derive(Debug, Args)]
pub(crate) struct ListActionsArgs {
    /// Registry filters.
    #[command(flatten)]
    filters: RegistryArgs,
    /// Cursor flags.
    #[command(flatten)]
    pages: PageArgs,
}

/// The arguments of `actions run`.
#[derive(Debug, Args)]
pub(crate) struct RunActionArgs {
    /// Action key, such as google_calendar-list-events.
    #[arg(value_name = "ACTION")]
    action: String,
    /// External user who owns the connected accounts.
    #[arg(long, value_name = "ID")]
    external_user_id: String,
    /// Configured props JSON file. Pass - to read standard input.
    #[arg(long, value_name = "FILE")]
    props: Option<PathBuf>,
    /// Action version. Pipedream uses the latest version when absent.
    #[arg(long, value_name = "VERSION")]
    version: Option<String>,
    /// Dynamic props ID from an earlier props reload.
    #[arg(long, value_name = "ID")]
    dynamic_props_id: Option<String>,
    /// File Stash selector: a stash ID, NEW, true, or false.
    #[arg(long, value_name = "STASH")]
    stash_id: Option<String>,
}

/// A validated action command.
#[derive(Debug, PartialEq)]
pub(crate) enum ActionsTask {
    /// List actions with fixed filters.
    List {
        /// The filters and the first page.
        request: ListRegistryRequest,
        /// The page budget.
        paging: Paging,
    },
    /// Run one action.
    Run(RunActionRequest),
}

/// The result of a run that Pipedream accepted for later completion.
#[derive(Debug, Serialize)]
struct AcceptedRun {
    status: RunStatus,
}

/// The state of an accepted run.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum RunStatus {
    /// Pipedream accepted the run and completes it later.
    Accepted,
}

impl Redact for AcceptedRun {
    fn redact(&mut self) {}
}

/// Validates an action command and reads its props input. Sends no request.
pub(crate) fn prepare(command: ActionsCommand, inputs: &dyn Inputs) -> Result<ActionsTask> {
    match command {
        ActionsCommand::List(args) => {
            let paging = args.pages.paging()?;
            let filters = args.filters.filters()?;
            let request = ListRegistryRequest {
                page: paging.page.clone(),
                query: filters.query,
                app: filters.app,
                registry: filters.registry,
            };
            Ok(ActionsTask::List { request, paging })
        }
        ActionsCommand::Run(args) => {
            let id = checked(ComponentKey::new(args.action), Argument::Action)?;
            let external_user_id = checked(
                ExternalUserId::new(args.external_user_id),
                Argument::ExternalUserId,
            )?;
            let configured_props = args
                .props
                .map(|path| read_props(inputs, &InputSource::from(path), Argument::Props))
                .transpose()?;
            Ok(ActionsTask::Run(RunActionRequest {
                id,
                version: args.version,
                external_user_id,
                configured_props,
                dynamic_props_id: args.dynamic_props_id,
                stash_id: args.stash_id.map(stash),
            }))
        }
    }
}

/// Runs a validated action command against the action trait object.
pub(crate) async fn run(
    task: ActionsTask,
    api: Arc<dyn ActionsApi + Send + Sync>,
    secrets: Secrets,
) -> Result<Output> {
    match task {
        ActionsTask::List { request, paging } => {
            let source = FilteredPageSource::new(request, move |request| {
                let api = api.clone();
                async move { api.list(request).await }
            });
            pages::list(Arc::new(source), paging, Target::Actions, secrets).await
        }
        ActionsTask::Run(request) => match api.run(request).await {
            Ok(response) => Ok(Document::redacted(response, secrets)?.into()),
            Err(pipedream_connect::error::Error::Accepted) => {
                let accepted = AcceptedRun {
                    status: RunStatus::Accepted,
                };
                Ok(Document::redacted(accepted, secrets)?.into())
            }
            Err(cause) => Err(Error::Api {
                target: Target::ActionRun,
                cause,
            }),
        },
    }
}

/// Maps `NEW`, `true`, and `false` to their selectors. Any other value is a stash ID.
fn stash(value: String) -> StashReference {
    match value.as_str() {
        "NEW" => StashReference::New,
        "true" => StashReference::Boolean(true),
        "false" => StashReference::Boolean(false),
        _ => StashReference::Id(value),
    }
}

#[cfg(test)]
#[path = "_tests_/actions/mod.rs"]
mod actions_tests;
