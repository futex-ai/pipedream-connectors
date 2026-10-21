//! Validated commands and their dispatch to the resource trait objects.

use std::sync::Arc;

use pipedream_connect::resources::ConnectResources;
use pipedream_connect_types::{CreateConnectTokenRequest, DeployTriggerRequest, ListUsageRequest};

use crate::cli::Command;
use crate::commands::accounts::AccountsTask;
use crate::commands::actions::ActionsTask;
use crate::commands::admin::{ProjectCommand, WebhookCommand};
use crate::commands::apps::AppsTask;
use crate::commands::categories::CategoriesCommand;
use crate::commands::components::ComponentsTask;
use crate::commands::proxy::args::ProxyTask;
use crate::commands::triggers::args::TriggersTask;
use crate::commands::triggers::deployed::DeployedTask;
use crate::commands::users::UsersTask;
use crate::commands::{
    accounts, actions, admin, apps, categories, components, proxy, tokens, triggers, users,
};
use crate::document::{Output, Secrets};
use crate::error::Result;
use crate::inputs::Inputs;

/// A command whose arguments and input files passed validation.
#[derive(Debug)]
pub(crate) enum Task {
    /// An app catalog command.
    Apps(AppsTask),
    /// An app category command.
    Categories(CategoriesCommand),
    /// A registry component command.
    Components(ComponentsTask),
    /// A registry action command.
    Actions(ActionsTask),
    /// A Connect token creation.
    Tokens(CreateConnectTokenRequest),
    /// A connected account command.
    Accounts(AccountsTask),
    /// An external user listing.
    Users(UsersTask),
    /// A proxy request.
    Proxy(ProxyTask),
    /// A command on deployed triggers.
    DeployedTriggers(DeployedTask),
    /// A registry trigger deployment.
    TriggerDeploy(DeployTriggerRequest),
    /// A project command.
    Project(ProjectCommand),
    /// A usage listing.
    Usage(ListUsageRequest),
    /// A project webhook command.
    Webhook(WebhookCommand),
}

/// Validates `command` and reads its input files. Sends no request.
pub(crate) fn prepare(command: Command, inputs: &dyn Inputs) -> Result<Task> {
    Ok(match command {
        Command::Apps(command) => Task::Apps(apps::prepare(command)?),
        Command::Categories(command) => Task::Categories(command),
        Command::Components(command) => Task::Components(components::prepare(command)?),
        Command::Actions(command) => Task::Actions(actions::prepare(command, inputs)?),
        Command::Tokens(command) => Task::Tokens(tokens::prepare(command)?),
        Command::Accounts(command) => Task::Accounts(accounts::prepare(command)?),
        Command::Users(command) => Task::Users(users::prepare(command)?),
        Command::Proxy(command) => Task::Proxy(proxy::args::prepare(command, inputs)?),
        Command::Triggers(command) => match triggers::args::prepare(command, inputs)? {
            TriggersTask::Deployed(task) => Task::DeployedTriggers(task),
            TriggersTask::Deploy(request) => Task::TriggerDeploy(request),
        },
        Command::Project(command) => Task::Project(command),
        Command::Usage(args) => Task::Usage(admin::usage_request(args)?),
        Command::Webhook(command) => Task::Webhook(command),
    })
}

/// Runs `task` with the one resource trait object that it needs.
pub(crate) async fn run(
    task: Task,
    resources: Arc<dyn ConnectResources + Send + Sync>,
    secrets: Secrets,
) -> Result<Output> {
    match task {
        Task::Apps(task) => apps::run(task, resources.apps(), secrets).await,
        Task::Categories(task) => categories::run(task, resources.app_categories(), secrets).await,
        Task::Components(task) => components::run(task, resources.components(), secrets).await,
        Task::Actions(task) => actions::run(task, resources.actions(), secrets).await,
        Task::Tokens(request) => tokens::run(request, resources.tokens()).await,
        Task::Accounts(task) => accounts::run(task, resources.accounts(), secrets).await,
        Task::Users(task) => users::run(task, resources.users(), secrets).await,
        Task::Proxy(task) => proxy::args::run(task, resources.proxy(), secrets).await,
        Task::DeployedTriggers(task) => {
            triggers::deployed::run(task, resources.deployed_triggers(), secrets).await
        }
        Task::TriggerDeploy(request) => {
            triggers::deploy::run(request, resources.triggers(), secrets).await
        }
        Task::Project(command) => admin::project(command, resources.projects(), secrets).await,
        Task::Usage(request) => admin::usage(request, resources.usage(), secrets).await,
        Task::Webhook(command) => {
            admin::webhook(command, resources.project_webhooks(), secrets).await
        }
    }
}
