//! Proxy arguments, their validation, and the proxy request.

use std::path::PathBuf;
use std::sync::Arc;

use clap::{Args, Subcommand};
use pipedream_connect::resources::proxy::{ProxyApi, ProxyMethod, ProxyRequest};
use pipedream_connect_types::{AccountId, AccountRef, ExternalUserId};

use crate::argument::Argument;
use crate::commands::proxy::headers;
use crate::commands::proxy::output::ProxyOutput;
use crate::document::{Document, Output, Secrets};
use crate::error::{Error, Result, called, checked};
use crate::inputs::{InputSource, Inputs};
use crate::target::Target;

/// The proxy commands, one for each HTTP method.
#[derive(Debug, Subcommand)]
pub(crate) enum ProxyCommand {
    /// Send a GET request through the Connect proxy.
    Get(ProxyArgs),
    /// Send a POST request through the Connect proxy. The CLI never retries it.
    Post(ProxyArgs),
    /// Send a PUT request through the Connect proxy. The CLI never retries it.
    Put(ProxyArgs),
    /// Send a PATCH request through the Connect proxy. The CLI never retries it.
    Patch(ProxyArgs),
    /// Send a DELETE request through the Connect proxy. The CLI never retries it.
    Delete(ProxyArgs),
}

/// The arguments of every proxy command.
#[derive(Debug, Args)]
pub(crate) struct ProxyArgs {
    /// Upstream URL: an HTTPS URL, or a path on the app's base URL that starts with /.
    #[arg(value_name = "URL")]
    url: String,
    /// External user who owns the account.
    #[arg(long, value_name = "ID")]
    external_user_id: String,
    /// Connected account that authenticates the request, such as apn_abc123.
    #[arg(long, value_name = "ACCOUNT")]
    account: String,
    /// Upstream header as 'Name: value'. Repeat the flag for more headers.
    #[arg(long = "header", value_name = "HEADER")]
    headers: Vec<String>,
    /// Request body file. Pass - to read standard input.
    #[arg(long, value_name = "FILE")]
    body: Option<PathBuf>,
}

/// A validated proxy request and the target for its failure text.
#[derive(Debug)]
pub(crate) struct ProxyTask {
    request: ProxyRequest,
    target: Target,
}

/// Validates the proxy arguments and reads the body input. Sends no request.
pub(crate) fn prepare(command: ProxyCommand, inputs: &dyn Inputs) -> Result<ProxyTask> {
    let (method, args) = match command {
        ProxyCommand::Get(args) => (ProxyMethod::Get, args),
        ProxyCommand::Post(args) => (ProxyMethod::Post, args),
        ProxyCommand::Put(args) => (ProxyMethod::Put, args),
        ProxyCommand::Patch(args) => (ProxyMethod::Patch, args),
        ProxyCommand::Delete(args) => (ProxyMethod::Delete, args),
    };
    let external_user_id = checked(
        ExternalUserId::new(args.external_user_id),
        Argument::ExternalUserId,
    )?;
    let account_id = checked(AccountId::new(args.account), Argument::AccountFlag)?;
    let headers = headers::parse(&args.headers)?;
    let mut request = ProxyRequest::new(
        AccountRef::new(external_user_id, account_id),
        method,
        args.url,
    )
    .with_headers(headers);
    if let Some(path) = args.body {
        let body = match inputs.read(&InputSource::from(path)) {
            Ok(body) => body,
            Err(cause) => {
                return Err(Error::UnreadableInput {
                    argument: Argument::Body,
                    cause,
                });
            }
        };
        request = request.with_body(body);
    }
    let target = match method {
        ProxyMethod::Get => Target::ProxyRead,
        ProxyMethod::Post | ProxyMethod::Put | ProxyMethod::Patch | ProxyMethod::Delete => {
            Target::ProxyWrite
        }
    };
    Ok(ProxyTask { request, target })
}

/// Sends the request once. Any upstream status is a result, like `curl` without `--fail`.
pub(crate) async fn run(
    task: ProxyTask,
    api: Arc<dyn ProxyApi + Send + Sync>,
    secrets: Secrets,
) -> Result<Output> {
    let response = called(api.send(task.request).await, task.target)?;
    Ok(Document::redacted(ProxyOutput::from(response), secrets)?.into())
}

#[cfg(test)]
#[path = "_tests_/args/mod.rs"]
mod args_tests;
