//! Tests for the command grammar, the command inventory, and the help text.

use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};

use crate::cli::{Cli, Command, EnvironmentArg};

#[test]
fn command_grammar_passes_the_clap_checks() {
    Cli::command().debug_assert();
}

#[test]
fn command_tree_holds_the_implemented_commands() {
    let mut leaves = Vec::new();
    collect(&Cli::command(), "", &mut leaves);
    leaves.sort();
    assert_eq!(
        leaves,
        [
            "actions list",
            "actions run",
            "apps get",
            "apps list",
            "categories list",
            "components get",
            "components list",
        ]
    );
}

fn collect(command: &clap::Command, prefix: &str, leaves: &mut Vec<String>) {
    for subcommand in command.get_subcommands() {
        if subcommand.get_name() == "help" {
            continue;
        }
        let path = format!("{prefix} {}", subcommand.get_name())
            .trim()
            .to_owned();
        if subcommand.has_subcommands() {
            collect(subcommand, &path, leaves);
        } else {
            leaves.push(path);
        }
    }
}

#[test]
fn global_flags_parse_after_the_subcommand() {
    let cli = Cli::try_parse_from([
        "pipedream-cli",
        "apps",
        "list",
        "--client-id",
        "id",
        "--project-id",
        "proj_Flag1",
        "--environment",
        "production",
        "--api-origin",
        "https://api.example.com",
        "--show-secrets",
    ])
    .unwrap();
    assert_eq!(cli.global.client_id.as_deref(), Some("id"));
    assert_eq!(cli.global.project_id.as_deref(), Some("proj_Flag1"));
    assert_eq!(cli.global.environment, Some(EnvironmentArg::Production));
    assert_eq!(
        cli.global.api_origin.as_deref(),
        Some("https://api.example.com")
    );
    assert!(cli.global.show_secrets);
    assert!(matches!(cli.command, Command::Apps(_)));
}

#[test]
fn credentials_that_start_with_a_hyphen_use_the_equals_form() {
    let cli = Cli::try_parse_from([
        "pipedream-cli",
        "--client-id=-id",
        "--client-secret=-secret",
        "categories",
        "list",
    ])
    .unwrap();
    assert_eq!(cli.global.client_id.as_deref(), Some("-id"));
    assert_eq!(
        cli.global
            .client_secret
            .as_ref()
            .map(|secret| secret.expose_secret()),
        Some("-secret")
    );
}

#[test]
fn a_flag_after_the_secret_flag_is_never_taken_as_the_secret() {
    let error = Cli::try_parse_from([
        "pipedream-cli",
        "categories",
        "list",
        "--client-secret",
        "--show-secrets",
    ])
    .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidValue);
}

#[test]
fn the_parsed_secret_never_reaches_debug_output() {
    let cli = Cli::try_parse_from([
        "pipedream-cli",
        "--client-secret",
        "debug-hidden-secret",
        "categories",
        "list",
    ])
    .unwrap();
    assert!(!format!("{cli:?}").contains("debug-hidden-secret"));
}

#[test]
fn secrets_are_redacted_unless_the_flag_is_present() {
    let cli = Cli::try_parse_from(["pipedream-cli", "categories", "list"]).unwrap();
    assert!(!cli.global.show_secrets);
}

#[test]
fn environment_flag_accepts_only_the_two_environments() {
    let error = Cli::try_parse_from([
        "pipedream-cli",
        "categories",
        "list",
        "--environment",
        "staging",
    ])
    .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidValue);
    assert_eq!(
        EnvironmentArg::Development.model(),
        pipedream_connect_types::Environment::Development
    );
    assert_eq!(
        EnvironmentArg::Production.model(),
        pipedream_connect_types::Environment::Production
    );
}

#[test]
fn help_names_each_variable_and_prefers_the_secret_variable() {
    let help = Cli::command().render_long_help().to_string();
    for variable in [
        "PIPEDREAM_CLIENT_ID",
        "PIPEDREAM_CLIENT_SECRET",
        "PIPEDREAM_PROJECT_ID",
        "PIPEDREAM_ENVIRONMENT",
        "PIPEDREAM_API_ORIGIN",
    ] {
        assert!(help.contains(variable), "help names {variable}");
    }
    assert!(help.contains("Prefer the variable"));
    assert!(help.contains("--client-secret=VALUE"));
    assert!(help.contains("--show-secrets"));
}

#[test]
fn group_without_a_command_shows_help_as_a_usage_error() {
    let error = Cli::try_parse_from(["pipedream-cli", "apps"]).unwrap_err();
    assert!(error.use_stderr());
    assert_eq!(error.exit_code(), 2);
}
