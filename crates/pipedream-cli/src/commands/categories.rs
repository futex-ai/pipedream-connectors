//! The app category command: `categories list`.

use std::sync::Arc;

use clap::Subcommand;
use pipedream_connect::resources::app_categories::AppCategoriesApi;

use crate::document::{Document, Output, Secrets};
use crate::error::{Result, called};
use crate::target::Target;

/// The app category commands.
#[derive(Debug, Eq, PartialEq, Subcommand)]
pub(crate) enum CategoriesCommand {
    /// List every app category. Pipedream returns all categories in one response.
    List,
}

/// Runs a category command against the category trait object.
pub(crate) async fn run(
    command: CategoriesCommand,
    api: Arc<dyn AppCategoriesApi + Send + Sync>,
    secrets: Secrets,
) -> Result<Output> {
    match command {
        CategoriesCommand::List => {
            let categories = called(api.list().await, Target::Categories)?;
            Ok(Document::redacted(categories, secrets)?.into())
        }
    }
}

#[cfg(test)]
#[path = "_tests_/categories_tests.rs"]
mod categories_tests;
