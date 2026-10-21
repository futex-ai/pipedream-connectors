//! App catalog commands: `apps list` and `apps get`.

use std::sync::Arc;

use clap::{Args, Subcommand, ValueEnum};
use pipedream_connect::pagination::FilteredPageSource;
use pipedream_connect::resources::apps::AppsApi;
use pipedream_connect_types::{AppCategoryId, AppSlug, AppSortKey, ListAppsRequest, SortDirection};

use crate::argument::Argument;
use crate::commands::pages::{self, PageArgs, Paging};
use crate::document::{Document, Output, Secrets};
use crate::error::{Result, called, checked};
use crate::target::Target;

/// The app catalog commands.
#[derive(Debug, Subcommand)]
pub(crate) enum AppsCommand {
    /// List apps in the Pipedream catalog.
    List(ListAppsArgs),
    /// Read one app by slug or ID.
    Get(GetAppArgs),
}

/// The arguments of `apps list`.
#[derive(Debug, Args)]
pub(crate) struct ListAppsArgs {
    /// Search text.
    #[arg(long, value_name = "TEXT")]
    query: Option<String>,
    /// Sort field.
    #[arg(long, value_enum)]
    sort_key: Option<SortKeyArg>,
    /// Sort direction.
    #[arg(long, value_enum)]
    sort_direction: Option<SortDirectionArg>,
    /// Category ID filter, such as appcat_abc123. Repeat the flag for more categories.
    #[arg(long = "category", value_name = "ID")]
    categories: Vec<String>,
    /// Keep apps that have components. Pass false to keep apps without them.
    #[arg(long, value_name = "BOOL", num_args = 0..=1, default_missing_value = "true")]
    has_components: Option<bool>,
    /// Keep apps that have actions. Pass false to keep apps without them.
    #[arg(long, value_name = "BOOL", num_args = 0..=1, default_missing_value = "true")]
    has_actions: Option<bool>,
    /// Keep apps that have triggers. Pass false to keep apps without them.
    #[arg(long, value_name = "BOOL", num_args = 0..=1, default_missing_value = "true")]
    has_triggers: Option<bool>,
    /// Cursor flags.
    #[command(flatten)]
    pages: PageArgs,
}

/// The arguments of `apps get`.
#[derive(Debug, Args)]
pub(crate) struct GetAppArgs {
    /// App slug or ID, such as google_calendar.
    #[arg(value_name = "APP")]
    app: String,
}

/// An app sort field.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum SortKeyArg {
    /// The display name.
    Name,
    /// The app slug.
    #[value(name = "name_slug")]
    NameSlug,
    /// The featured weight.
    #[value(name = "featured_weight")]
    FeaturedWeight,
}

/// An app sort direction.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum SortDirectionArg {
    /// Ascending order.
    Asc,
    /// Descending order.
    Desc,
}

/// A validated app command.
#[derive(Debug, PartialEq)]
pub(crate) enum AppsTask {
    /// List apps with fixed filters.
    List {
        /// The filters and the first page.
        request: ListAppsRequest,
        /// The page budget.
        paging: Paging,
    },
    /// Read one app.
    Get(AppSlug),
}

/// Validates an app command. Sends no request.
pub(crate) fn prepare(command: AppsCommand) -> Result<AppsTask> {
    match command {
        AppsCommand::List(args) => {
            let paging = args.pages.paging()?;
            let category_ids = args
                .categories
                .into_iter()
                .map(|id| checked(AppCategoryId::new(id), Argument::Category))
                .collect::<Result<Vec<_>>>()?;
            let request = ListAppsRequest {
                page: paging.page.clone(),
                query: args.query,
                sort_key: args.sort_key.map(|key| match key {
                    SortKeyArg::Name => AppSortKey::Name,
                    SortKeyArg::NameSlug => AppSortKey::NameSlug,
                    SortKeyArg::FeaturedWeight => AppSortKey::FeaturedWeight,
                }),
                sort_direction: args.sort_direction.map(|direction| match direction {
                    SortDirectionArg::Asc => SortDirection::Asc,
                    SortDirectionArg::Desc => SortDirection::Desc,
                }),
                category_ids,
                has_components: args.has_components,
                has_actions: args.has_actions,
                has_triggers: args.has_triggers,
            };
            Ok(AppsTask::List { request, paging })
        }
        AppsCommand::Get(args) => Ok(AppsTask::Get(checked(
            AppSlug::new(args.app),
            Argument::App,
        )?)),
    }
}

/// Runs a validated app command against the app trait object.
pub(crate) async fn run(
    task: AppsTask,
    api: Arc<dyn AppsApi + Send + Sync>,
    secrets: Secrets,
) -> Result<Output> {
    match task {
        AppsTask::List { request, paging } => {
            let source = FilteredPageSource::new(request, move |request| {
                let api = api.clone();
                async move { api.list(request).await }
            });
            pages::list(Arc::new(source), paging, Target::Apps, secrets).await
        }
        AppsTask::Get(slug) => {
            let app = called(api.get(slug).await, Target::App)?;
            Ok(Document::redacted(app, secrets)?.into())
        }
    }
}

#[cfg(test)]
#[path = "_tests_/apps_tests.rs"]
mod apps_tests;
