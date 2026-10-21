//! The external user command: `users list`.

use std::sync::Arc;

use clap::{Args, Subcommand};
use pipedream_connect::pagination::FilteredPageSource;
use pipedream_connect::resources::users::UsersApi;
use pipedream_connect_types::ListUsersRequest;

use crate::commands::pages::{self, PageArgs, Paging};
use crate::document::{Output, Secrets};
use crate::error::Result;
use crate::target::Target;

/// The external user commands.
#[derive(Debug, Subcommand)]
pub(crate) enum UsersCommand {
    /// List the external users of the project.
    List(ListUsersArgs),
}

/// The arguments of `users list`.
#[derive(Debug, Args)]
pub(crate) struct ListUsersArgs {
    /// Search text that matches part of an external user ID.
    #[arg(long, value_name = "TEXT")]
    query: Option<String>,
    /// Cursor flags.
    #[command(flatten)]
    pages: PageArgs,
}

/// A validated user listing.
#[derive(Debug, PartialEq)]
pub(crate) struct UsersTask {
    /// The filters and the first page.
    request: ListUsersRequest,
    /// The page budget.
    paging: Paging,
}

/// Validates a user command. Sends no request.
pub(crate) fn prepare(command: UsersCommand) -> Result<UsersTask> {
    let UsersCommand::List(args) = command;
    let paging = args.pages.paging()?;
    Ok(UsersTask {
        request: ListUsersRequest {
            page: paging.page.clone(),
            query: args.query,
        },
        paging,
    })
}

/// Lists external users through the user trait object.
pub(crate) async fn run(
    task: UsersTask,
    api: Arc<dyn UsersApi + Send + Sync>,
    secrets: Secrets,
) -> Result<Output> {
    let source = FilteredPageSource::new(task.request, move |request| {
        let api = api.clone();
        async move { api.list(request).await }
    });
    pages::list(Arc::new(source), task.paging, Target::Users, secrets).await
}

#[cfg(test)]
#[path = "_tests_/users_tests.rs"]
mod users_tests;
