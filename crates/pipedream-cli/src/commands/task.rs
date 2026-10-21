//! Validated commands and their dispatch to the resource trait objects.

use std::sync::Arc;

use pipedream_connect::resources::ConnectResources;

use crate::cli::Command;
use crate::commands::actions::{self, ActionsTask};
use crate::commands::apps::{self, AppsTask};
use crate::commands::categories::{self, CategoriesCommand};
use crate::commands::components::{self, ComponentsTask};
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
}

/// Validates `command` and reads its input files. Sends no request.
pub(crate) fn prepare(command: Command, inputs: &dyn Inputs) -> Result<Task> {
    Ok(match command {
        Command::Apps(command) => Task::Apps(apps::prepare(command)?),
        Command::Categories(command) => Task::Categories(command),
        Command::Components(command) => Task::Components(components::prepare(command)?),
        Command::Actions(command) => Task::Actions(actions::prepare(command, inputs)?),
    })
}

/// Runs `task` with the one resource trait object that its group needs.
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
    }
}
