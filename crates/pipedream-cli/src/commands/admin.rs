//! Administration commands: `project info`, `usage`, and `webhook get`.

use std::sync::Arc;

use chrono::DateTime;
use clap::{Args, Subcommand};
use pipedream_connect::resources::project_webhooks::ProjectWebhooksApi;
use pipedream_connect::resources::projects::ProjectsApi;
use pipedream_connect::resources::usage::UsageApi;
use pipedream_connect_types::ListUsageRequest;

use crate::argument::Argument;
use crate::document::{Output, Secrets};
use crate::error::{Error, Result, called};
use crate::target::Target;

/// The project commands.
#[derive(Debug, Eq, PartialEq, Subcommand)]
pub(crate) enum ProjectCommand {
    /// List the apps enabled in the project environment.
    Info,
}

/// The project webhook commands.
#[derive(Debug, Eq, PartialEq, Subcommand)]
pub(crate) enum WebhookCommand {
    /// Read the webhook of the project environment.
    Get,
}

/// The arguments of `usage`.
#[derive(Debug, Args)]
pub(crate) struct UsageArgs {
    /// Start of the usage window: Unix seconds or an RFC 3339 time.
    #[arg(long, value_name = "TIME", allow_negative_numbers = true)]
    start: String,
    /// End of the usage window: Unix seconds or an RFC 3339 time.
    #[arg(long, value_name = "TIME", allow_negative_numbers = true)]
    end: String,
}

/// Validates the usage window. Sends no request.
pub(crate) fn usage_request(args: UsageArgs) -> Result<ListUsageRequest> {
    Ok(ListUsageRequest {
        start_ts: seconds(&args.start, Argument::Start)?,
        end_ts: seconds(&args.end, Argument::End)?,
    })
}

/// Reads Unix seconds, or converts an RFC 3339 time to Unix seconds.
fn seconds(text: &str, argument: Argument) -> Result<i64> {
    if let Ok(seconds) = text.parse::<i64>() {
        return Ok(seconds);
    }
    match DateTime::parse_from_rfc3339(text) {
        Ok(time) => Ok(time.timestamp()),
        Err(_) => Err(Error::InvalidTime { argument }),
    }
}

/// Lists the apps enabled in the project environment.
pub(crate) async fn project(
    command: ProjectCommand,
    api: Arc<dyn ProjectsApi + Send + Sync>,
    secrets: Secrets,
) -> Result<Output> {
    let ProjectCommand::Info = command;
    let info = called(api.info().await, Target::ProjectInfo)?;
    Output::redacted(Target::ProjectInfo, info, secrets)
}

/// Lists Connect usage for the validated time window.
pub(crate) async fn usage(
    request: ListUsageRequest,
    api: Arc<dyn UsageApi + Send + Sync>,
    secrets: Secrets,
) -> Result<Output> {
    let records = called(api.list(request).await, Target::Usage)?;
    Output::redacted(Target::Usage, records, secrets)
}

/// Reads the project environment webhook. A project without one prints `"data": null`.
pub(crate) async fn webhook(
    command: WebhookCommand,
    api: Arc<dyn ProjectWebhooksApi + Send + Sync>,
    secrets: Secrets,
) -> Result<Output> {
    let WebhookCommand::Get = command;
    let webhook = called(api.get().await, Target::Webhook)?;
    Output::redacted(Target::Webhook, webhook, secrets)
}

#[cfg(test)]
#[path = "_tests_/admin_tests.rs"]
mod admin_tests;
