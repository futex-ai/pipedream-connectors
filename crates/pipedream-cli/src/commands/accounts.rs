//! Connected account commands: `accounts list` and `accounts get`.

use std::sync::Arc;

use clap::{Args, Subcommand};
use pipedream_connect::pagination::FilteredPageSource;
use pipedream_connect::resources::accounts::AccountsApi;
use pipedream_connect_types::{
    AccountId, AccountRef, AppSlug, ExternalUserId, GetAccountRequest, ListAccountsRequest,
    OauthAppId,
};

use crate::argument::Argument;
use crate::commands::pages::{self, PageArgs, Paging};
use crate::document::{Document, Output, Secrets};
use crate::error::{Result, called, checked};
use crate::target::Target;

/// The connected account commands.
#[derive(Debug, Subcommand)]
pub(crate) enum AccountsCommand {
    /// List connected accounts.
    List(ListAccountsArgs),
    /// Read one connected account.
    Get(GetAccountArgs),
}

/// The arguments of `accounts list`.
#[derive(Debug, Args)]
pub(crate) struct ListAccountsArgs {
    /// Keep the accounts of this external user.
    #[arg(long, value_name = "ID")]
    external_user_id: Option<String>,
    /// Keep the accounts of this app, such as google_calendar.
    #[arg(long, value_name = "APP")]
    app: Option<String>,
    /// Keep the accounts of this custom OAuth app, such as oa_abc123.
    #[arg(long, value_name = "ID")]
    oauth_app_id: Option<String>,
    /// Ask Pipedream for the account credentials. They print only with --show-secrets.
    #[arg(long)]
    include_credentials: bool,
    /// Cursor flags.
    #[command(flatten)]
    pages: PageArgs,
}

/// The arguments of `accounts get`.
#[derive(Debug, Args)]
pub(crate) struct GetAccountArgs {
    /// Account ID, such as apn_abc123.
    #[arg(value_name = "ACCOUNT")]
    account: String,
    /// Check that the account belongs to this external user.
    #[arg(long, value_name = "ID")]
    external_user_id: Option<String>,
    /// Ask Pipedream for the account credentials. They print only with --show-secrets.
    #[arg(long)]
    include_credentials: bool,
}

/// A validated account command.
#[derive(Debug, PartialEq)]
pub(crate) enum AccountsTask {
    /// List accounts with fixed filters.
    List {
        /// The filters and the first page.
        request: ListAccountsRequest,
        /// The page budget.
        paging: Paging,
    },
    /// Read one account of the project.
    Get(AccountId, GetAccountRequest),
    /// Read one account after an ownership check.
    GetOwned(AccountRef, GetAccountRequest),
}

/// Validates an account command. Sends no request.
pub(crate) fn prepare(command: AccountsCommand) -> Result<AccountsTask> {
    match command {
        AccountsCommand::List(args) => {
            let paging = args.pages.paging()?;
            let request = ListAccountsRequest {
                page: paging.page.clone(),
                external_user_id: external_user(args.external_user_id)?,
                app: args
                    .app
                    .map(|app| checked(AppSlug::new(app), Argument::AppFilter))
                    .transpose()?,
                oauth_app_id: args
                    .oauth_app_id
                    .map(|id| checked(OauthAppId::new(id), Argument::OauthAppId))
                    .transpose()?,
                include_credentials: args.include_credentials.then_some(true),
            };
            Ok(AccountsTask::List { request, paging })
        }
        AccountsCommand::Get(args) => {
            let account_id = checked(AccountId::new(args.account), Argument::Account)?;
            let request = GetAccountRequest {
                include_credentials: args.include_credentials.then_some(true),
            };
            Ok(match external_user(args.external_user_id)? {
                Some(user) => AccountsTask::GetOwned(AccountRef::new(user, account_id), request),
                None => AccountsTask::Get(account_id, request),
            })
        }
    }
}

fn external_user(value: Option<String>) -> Result<Option<ExternalUserId>> {
    value
        .map(|user| checked(ExternalUserId::new(user), Argument::ExternalUserId))
        .transpose()
}

/// Runs a validated account command against the account trait object.
pub(crate) async fn run(
    task: AccountsTask,
    api: Arc<dyn AccountsApi + Send + Sync>,
    secrets: Secrets,
) -> Result<Output> {
    match task {
        AccountsTask::List { request, paging } => {
            let source = FilteredPageSource::new(request, move |request| {
                let api = api.clone();
                async move { api.list(request).await }
            });
            pages::list(Arc::new(source), paging, Target::Accounts, secrets).await
        }
        AccountsTask::Get(account_id, request) => {
            let account = called(api.get(account_id, request).await, Target::Account)?;
            Ok(Document::redacted(account, secrets)?.into())
        }
        AccountsTask::GetOwned(account, request) => {
            let account = called(api.get_owned(account, request).await, Target::Account)?;
            Ok(Document::redacted(account, secrets)?.into())
        }
    }
}

#[cfg(test)]
#[path = "_tests_/accounts_tests.rs"]
mod accounts_tests;
