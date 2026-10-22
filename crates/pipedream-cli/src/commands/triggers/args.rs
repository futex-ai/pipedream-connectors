//! The arguments of the trigger commands and the split between their trait objects.

use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};
use pipedream_connect_types::{DeployTriggerRequest, EmitterType};

use crate::commands::pages::PageArgs;
use crate::commands::triggers::{deploy, deployed};
use crate::error::Result;
use crate::inputs::Inputs;

/// The trigger commands.
#[derive(Debug, Subcommand)]
pub(crate) enum TriggersCommand {
    /// List the deployed triggers of an external user.
    List(ListTriggersArgs),
    /// Read one deployed trigger.
    Get(TriggerArgs),
    /// Deploy a registry trigger for an external user. The CLI never retries it.
    Deploy(DeployTriggerArgs),
    /// Delete one deployed trigger. The CLI never retries it.
    Delete(DeleteTriggerArgs),
    /// List the recent events of one deployed trigger.
    Events(TriggerEventsArgs),
}

/// The arguments of `triggers list`.
#[derive(Debug, Args)]
pub(crate) struct ListTriggersArgs {
    /// External user who owns the deployed triggers.
    #[arg(long, value_name = "ID")]
    pub(super) external_user_id: String,
    /// Keep the triggers of this emitter kind.
    #[arg(long, value_enum)]
    pub(super) emitter_type: Option<EmitterTypeArg>,
    /// Cursor flags.
    #[command(flatten)]
    pub(super) pages: PageArgs,
}

/// The deployed trigger and its owner.
#[derive(Debug, Args)]
pub(crate) struct TriggerArgs {
    /// Deployed trigger ID, which starts with dc_, hi_, or ti_.
    #[arg(value_name = "TRIGGER")]
    pub(super) trigger: String,
    /// External user who owns the deployed trigger.
    #[arg(long, value_name = "ID")]
    pub(super) external_user_id: String,
}

/// The arguments of `triggers delete`.
#[derive(Debug, Args)]
pub(crate) struct DeleteTriggerArgs {
    /// The deployed trigger and its owner.
    #[command(flatten)]
    pub(super) trigger: TriggerArgs,
    /// Delete the trigger even when its deactivation hooks fail.
    #[arg(long)]
    pub(super) ignore_hook_errors: bool,
    /// Succeed when the trigger does not exist.
    #[arg(long)]
    pub(super) missing_ok: bool,
}

/// The arguments of `triggers events`.
#[derive(Debug, Args)]
pub(crate) struct TriggerEventsArgs {
    /// The deployed trigger and its owner.
    #[command(flatten)]
    pub(super) trigger: TriggerArgs,
    /// Events to read, from 1 to 100. The default is 20, the Pipedream default.
    #[arg(long, value_name = "COUNT", allow_negative_numbers = true)]
    pub(super) limit: Option<String>,
}

/// The arguments of `triggers deploy`.
#[derive(Debug, Args)]
pub(crate) struct DeployTriggerArgs {
    /// Trigger key, such as google_calendar-new-or-updated-event-instant.
    #[arg(value_name = "COMPONENT")]
    pub(super) component: String,
    /// External user who owns the connected accounts.
    #[arg(long, value_name = "ID")]
    pub(super) external_user_id: String,
    /// Configured props JSON file. Pass - to read standard input.
    #[arg(long, value_name = "FILE")]
    pub(super) props: Option<PathBuf>,
    /// Trigger version. Pipedream uses the latest version when absent.
    #[arg(long, value_name = "VERSION")]
    pub(super) version: Option<String>,
    /// Dynamic props ID from an earlier props reload.
    #[arg(long, value_name = "ID")]
    pub(super) dynamic_props_id: Option<String>,
    /// URL that receives the trigger events.
    #[arg(long, value_name = "URL")]
    pub(super) webhook_url: Option<String>,
    /// Workflow that receives the trigger events.
    #[arg(long, value_name = "ID")]
    pub(super) workflow_id: Option<String>,
    /// Let the trigger emit events while it deploys. Pass false to stop them.
    #[arg(long, value_name = "BOOL", num_args = 0..=1, default_missing_value = "true")]
    pub(super) emit_on_deploy: Option<bool>,
}

/// A deployed trigger emitter kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum EmitterTypeArg {
    /// Email sources.
    Email,
    /// HTTP sources.
    Http,
    /// Event sources.
    Source,
    /// Timer sources.
    Timer,
}

impl EmitterTypeArg {
    /// Returns the SDK emitter type for this value.
    pub(super) fn model(self) -> EmitterType {
        match self {
            Self::Email => EmitterType::Email,
            Self::Http => EmitterType::Http,
            Self::Source => EmitterType::Source,
            Self::Timer => EmitterType::Timer,
        }
    }
}

/// A validated trigger command, split by the trait object that runs it.
#[derive(Debug, PartialEq)]
pub(crate) enum TriggersTask {
    /// A command on deployed triggers.
    Deployed(deployed::DeployedTask),
    /// A deployment of a registry trigger.
    Deploy(DeployTriggerRequest),
}

/// Validates a trigger command and reads its props input. Sends no request.
pub(crate) fn prepare(command: TriggersCommand, inputs: &dyn Inputs) -> Result<TriggersTask> {
    Ok(match command {
        TriggersCommand::Deploy(args) => TriggersTask::Deploy(deploy::prepare(args, inputs)?),
        TriggersCommand::List(args) => TriggersTask::Deployed(deployed::list(args)?),
        TriggersCommand::Get(args) => TriggersTask::Deployed(deployed::get(args)?),
        TriggersCommand::Delete(args) => TriggersTask::Deployed(deployed::delete(args)?),
        TriggersCommand::Events(args) => TriggersTask::Deployed(deployed::events(args)?),
    })
}
