//! The CLI pipeline: parse, resolve settings, validate input, connect, run, and report.

use std::ffi::OsString;
use std::sync::Arc;

use async_trait::async_trait;
use clap::Parser;

use crate::cli::{Cli, Command};
use crate::commands::task;
use crate::connector::Connector;
use crate::document::{Output, Secrets};
use crate::error::{Error, Result};
use crate::inputs::Inputs;
use crate::report::exit::Exit;
use crate::report::outcome;
use crate::settings::{self, DEFAULT_ORIGIN, Settings};
use crate::terminal::Terminal;
use crate::variables::Variables;

/// One CLI invocation, from its arguments to its exit status.
#[async_trait]
pub(crate) trait Application: Send + Sync {
    /// Runs one command, writes its result or its failure, and returns the exit status.
    async fn run(&self, args: Vec<OsString>, terminal: &mut Terminal<'_>) -> Exit;
}

/// The CLI with injected environment, input, and client boundaries.
pub(crate) struct App {
    variables: Arc<dyn Variables + Send + Sync>,
    inputs: Arc<dyn Inputs + Send + Sync>,
    connector: Arc<dyn Connector + Send + Sync>,
}

impl App {
    /// Injects the process boundaries without reading them.
    pub(crate) fn new(
        variables: Arc<dyn Variables + Send + Sync>,
        inputs: Arc<dyn Inputs + Send + Sync>,
        connector: Arc<dyn Connector + Send + Sync>,
    ) -> Self {
        Self {
            variables,
            inputs,
            connector,
        }
    }

    /// Validates the command and its input, then builds the client once and runs it.
    async fn execute(
        &self,
        settings: &Settings,
        command: Command,
        secrets: Secrets,
    ) -> Result<Output> {
        let task = task::prepare(command, self.inputs.as_ref())?;
        let resources = match self.connector.connect(settings) {
            Ok(resources) => resources,
            Err(cause) => return Err(Error::Client(cause)),
        };
        task::run(task, resources, secrets).await
    }
}

#[async_trait]
impl Application for App {
    async fn run(&self, args: Vec<OsString>, terminal: &mut Terminal<'_>) -> Exit {
        let cli = match Cli::try_parse_from(args) {
            Ok(cli) => cli,
            Err(error) => return outcome::usage(&error, terminal),
        };
        let secrets = if cli.global.show_secrets {
            Secrets::Show
        } else {
            Secrets::Redact
        };
        let settings = match settings::resolve(&cli.global, self.variables.as_ref()) {
            Ok(settings) => settings,
            Err(error) => return outcome::finish(Err(error), DEFAULT_ORIGIN, terminal),
        };
        if let Some(origin) = settings.custom_origin() {
            outcome::origin_note(origin, terminal);
        }
        let result = self.execute(&settings, cli.command, secrets).await;
        outcome::finish(result, settings.origin(), terminal)
    }
}

#[cfg(test)]
#[path = "_tests_/app_tests.rs"]
mod app_tests;
