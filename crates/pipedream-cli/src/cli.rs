//! Command-line grammar: the global configuration flags and the command tree.

use std::convert::Infallible;

use clap::{Args, Parser, Subcommand, ValueEnum};
use pipedream_connect_types::{Environment, SecretString};

use crate::commands::actions::ActionsCommand;
use crate::commands::apps::AppsCommand;
use crate::commands::categories::CategoriesCommand;
use crate::commands::components::ComponentsCommand;

/// Run Pipedream Connect commands and print each result as JSON.
#[derive(Debug, Parser)]
#[command(name = "pipedream-cli", version, arg_required_else_help = true)]
pub(crate) struct Cli {
    /// Connection settings and output options.
    #[command(flatten)]
    pub(crate) global: GlobalArgs,
    /// The selected command.
    #[command(subcommand)]
    pub(crate) command: Command,
}

/// Connection settings and output options that every command accepts.
#[derive(Debug, Args)]
pub(crate) struct GlobalArgs {
    /// Pipedream OAuth client ID. Reads PIPEDREAM_CLIENT_ID when absent.
    #[arg(long, global = true, value_name = "ID", help_heading = "Configuration")]
    pub(crate) client_id: Option<String>,
    /// Pipedream OAuth client secret. Reads PIPEDREAM_CLIENT_SECRET when absent.
    /// Prefer the variable, because shell history and process lists can show flag values.
    /// Pass a secret that starts with a hyphen as --client-secret=VALUE.
    #[arg(
        long,
        global = true,
        value_name = "SECRET",
        value_parser = secret,
        help_heading = "Configuration"
    )]
    pub(crate) client_secret: Option<SecretString>,
    /// Connect project ID, such as proj_abc123. Reads PIPEDREAM_PROJECT_ID when absent.
    #[arg(long, global = true, value_name = "ID", help_heading = "Configuration")]
    pub(crate) project_id: Option<String>,
    /// Project environment. Reads PIPEDREAM_ENVIRONMENT when absent.
    #[arg(long, global = true, value_enum, help_heading = "Configuration")]
    pub(crate) environment: Option<EnvironmentArg>,
    /// The API origin flag. Its help text names the default origin.
    #[arg(
        long,
        global = true,
        value_name = "URL",
        help = "API origin. Reads PIPEDREAM_API_ORIGIN when absent, then uses https://api.pipedream.com",
        help_heading = "Configuration"
    )]
    pub(crate) api_origin: Option<String>,
    /// Whether the output shows credential-class values instead of the redaction marker.
    #[arg(
        long,
        global = true,
        help = "Print credentials in the output instead of [REDACTED]",
        help_heading = "Output"
    )]
    pub(crate) show_secrets: bool,
}

/// Wraps the client secret flag, so that `Debug` output never shows it.
fn secret(value: &str) -> Result<SecretString, Infallible> {
    Ok(SecretString::new(value))
}

/// A Connect project environment.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum EnvironmentArg {
    /// The development environment.
    Development,
    /// The production environment.
    Production,
}

impl EnvironmentArg {
    /// Returns the SDK environment for this value.
    pub(crate) fn model(self) -> Environment {
        match self {
            Self::Development => Environment::Development,
            Self::Production => Environment::Production,
        }
    }
}

/// The command groups.
#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// List and read apps in the Pipedream catalog.
    #[command(subcommand)]
    Apps(AppsCommand),
    /// List app categories.
    #[command(subcommand)]
    Categories(CategoriesCommand),
    /// List and read registry components.
    #[command(subcommand)]
    Components(ComponentsCommand),
    /// List and run registry actions.
    #[command(subcommand)]
    Actions(ActionsCommand),
}

#[cfg(test)]
#[path = "_tests_/cli_tests.rs"]
mod cli_tests;
