//! Registry component commands: `components list` and `components get`.

use std::sync::Arc;

use clap::{Args, Subcommand};
use pipedream_connect::pagination::FilteredPageSource;
use pipedream_connect::resources::components::ComponentsApi;
use pipedream_connect_types::{ComponentKey, GetComponentRequest, ListComponentsRequest};

use crate::argument::Argument;
use crate::commands::pages::{self, PageArgs, Paging};
use crate::commands::registry::{ComponentTypeArg, RegistryArgs};
use crate::document::{Output, Secrets};
use crate::error::{Result, called, checked};
use crate::target::Target;

/// The registry component commands.
#[derive(Debug, Subcommand)]
pub(crate) enum ComponentsCommand {
    /// List registry components.
    List(ListComponentsArgs),
    /// Read one registry component and its configurable props.
    Get(GetComponentArgs),
}

/// The arguments of `components list`.
#[derive(Debug, Args)]
pub(crate) struct ListComponentsArgs {
    /// Registry filters.
    #[command(flatten)]
    filters: RegistryArgs,
    /// Component kind.
    #[arg(long = "type", value_enum)]
    component_type: Option<ComponentTypeArg>,
    /// Cursor flags.
    #[command(flatten)]
    pages: PageArgs,
}

/// The arguments of `components get`.
#[derive(Debug, Args)]
pub(crate) struct GetComponentArgs {
    /// Component key, such as google_calendar-list-events.
    #[arg(value_name = "COMPONENT")]
    component: String,
    /// Component version. Pipedream uses the latest version when absent.
    #[arg(long, value_name = "VERSION")]
    version: Option<String>,
}

/// A validated component command.
#[derive(Debug, PartialEq)]
pub(crate) enum ComponentsTask {
    /// List components with fixed filters.
    List {
        /// The filters and the first page.
        request: ListComponentsRequest,
        /// The page budget.
        paging: Paging,
    },
    /// Read one component.
    Get {
        /// The component key.
        key: ComponentKey,
        /// The version query.
        request: GetComponentRequest,
    },
}

/// Validates a component command. Sends no request.
pub(crate) fn prepare(command: ComponentsCommand) -> Result<ComponentsTask> {
    match command {
        ComponentsCommand::List(args) => {
            let paging = args.pages.paging()?;
            let filters = args.filters.filters()?;
            let request = ListComponentsRequest {
                page: paging.page.clone(),
                query: filters.query,
                app: filters.app,
                registry: filters.registry,
                component_type: args.component_type.map(ComponentTypeArg::model),
            };
            Ok(ComponentsTask::List { request, paging })
        }
        ComponentsCommand::Get(args) => Ok(ComponentsTask::Get {
            key: checked(ComponentKey::new(args.component), Argument::Component)?,
            request: GetComponentRequest {
                version: args.version,
            },
        }),
    }
}

/// Runs a validated component command against the component trait object.
pub(crate) async fn run(
    task: ComponentsTask,
    api: Arc<dyn ComponentsApi + Send + Sync>,
    secrets: Secrets,
) -> Result<Output> {
    match task {
        ComponentsTask::List { request, paging } => {
            let source = FilteredPageSource::new(request, move |request| {
                let api = api.clone();
                async move { api.list(request).await }
            });
            pages::list(Arc::new(source), paging, Target::Components, secrets).await
        }
        ComponentsTask::Get { key, request } => {
            let component = called(api.get(key, request).await, Target::Component)?;
            Output::redacted(Target::Component, component, secrets)
        }
    }
}

#[cfg(test)]
#[path = "_tests_/components_tests.rs"]
mod components_tests;
