//! The Connect token command: `tokens create`.

use std::sync::Arc;

use clap::{Args, Subcommand};
use pipedream_connect::resources::tokens::TokensApi;
use pipedream_connect_types::{CreateConnectTokenRequest, ExternalUserId, UrlField};

use crate::argument::Argument;
use crate::commands::numbers::bounded;
use crate::document::{Document, Output};
use crate::error::{Error, Result, called, checked};
use crate::target::Target;

/// The Connect token commands.
#[derive(Debug, Subcommand)]
pub(crate) enum TokensCommand {
    /// Create a Connect token and its Connect Link URL for an external user.
    Create(CreateTokenArgs),
}

/// The arguments of `tokens create`.
#[derive(Debug, Args)]
pub(crate) struct CreateTokenArgs {
    /// External user who connects accounts with the token.
    #[arg(long, value_name = "ID")]
    external_user_id: String,
    /// Token lifetime in seconds, from 1 to 14400. The default is 14400.
    #[arg(long, value_name = "SECONDS", allow_negative_numbers = true)]
    expires_in: Option<String>,
    /// Browser origin that may use the token. Repeat the flag for more origins.
    #[arg(long = "allowed-origin", value_name = "ORIGIN")]
    allowed_origins: Vec<String>,
    /// HTTPS URL that Connect opens after a successful connection.
    #[arg(long, value_name = "URL")]
    success_redirect_uri: Option<String>,
    /// HTTPS URL that Connect opens after a failed connection.
    #[arg(long, value_name = "URL")]
    error_redirect_uri: Option<String>,
    /// HTTPS URL that receives the connection webhooks.
    #[arg(long, value_name = "URL")]
    webhook_uri: Option<String>,
    /// Connect scope. Pipedream uses connect:* when absent.
    #[arg(long, value_name = "SCOPE")]
    scope: Option<String>,
    /// Let the external user grant scopes step by step. Pass false to forbid it.
    #[arg(long, value_name = "BOOL", num_args = 0..=1, default_missing_value = "true")]
    allow_progressive_scopes: Option<bool>,
}

/// Validates the token request. Sends no request.
pub(crate) fn prepare(command: TokensCommand) -> Result<CreateConnectTokenRequest> {
    let TokensCommand::Create(args) = command;
    let external_user_id = checked(
        ExternalUserId::new(args.external_user_id),
        Argument::ExternalUserId,
    )?;
    let mut request = CreateConnectTokenRequest::new(external_user_id);
    if let Some(expires_in) = args.expires_in {
        request = checked(
            request.with_expires_in(bounded(&expires_in)),
            Argument::ExpiresIn,
        )?;
    }
    request.allowed_origins = args.allowed_origins;
    request.success_redirect_uri = args.success_redirect_uri;
    request.error_redirect_uri = args.error_redirect_uri;
    request.webhook_uri = args.webhook_uri;
    request.scope = args.scope;
    request.allow_progressive_scopes = args.allow_progressive_scopes;
    match request.validate() {
        Ok(()) => Ok(request),
        Err(cause) => {
            let argument = match &cause {
                pipedream_connect_types::Error::InvalidHttpsUrl { field } => match field {
                    UrlField::SuccessRedirectUri => Argument::SuccessRedirectUri,
                    UrlField::ErrorRedirectUri => Argument::ErrorRedirectUri,
                    UrlField::WebhookUri => Argument::WebhookUri,
                },
                _ => Argument::ExpiresIn,
            };
            Err(Error::InvalidArgument { argument, cause })
        }
    }
}

/// Creates the token and prints it, because this command exists to produce it.
pub(crate) async fn run(
    request: CreateConnectTokenRequest,
    api: Arc<dyn TokensApi + Send + Sync>,
) -> Result<Output> {
    let response = called(api.create(request).await, Target::TokenCreate)?;
    Ok(Document::revealed(&response)?.into())
}

#[cfg(test)]
#[path = "_tests_/tokens_tests.rs"]
mod tokens_tests;
