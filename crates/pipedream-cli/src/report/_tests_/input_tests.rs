//! Tests for the exit status and the text of configuration and input failures.

use std::io;

use pipedream_connect_types::{IdentifierKind, UrlField, ValidationField};

use crate::argument::Argument;
use crate::error::{Error, JsonProblem};
use crate::report::exit::Exit;
use crate::report::failure::{Failure, describe};
use crate::settings::{Setting, Supply};

fn usage(message: &str) -> Failure {
    Failure::new(Exit::Usage, message)
}

fn argument(argument: Argument, cause: pipedream_connect_types::Error) -> Failure {
    describe(&Error::InvalidArgument { argument, cause })
}

#[test]
fn missing_settings_name_the_variable_and_the_flag() {
    let cases = [
        (
            Setting::ClientId,
            "The client ID is missing. Set PIPEDREAM_CLIENT_ID or pass --client-id.",
        ),
        (
            Setting::ClientSecret,
            "The client secret is missing. Set PIPEDREAM_CLIENT_SECRET.",
        ),
        (
            Setting::ProjectId,
            "The project ID is missing. Set PIPEDREAM_PROJECT_ID or pass --project-id.",
        ),
        (
            Setting::Environment,
            "The environment is missing. Set PIPEDREAM_ENVIRONMENT to development or \
             production, or pass --environment.",
        ),
        (
            Setting::ApiOrigin,
            "The API origin is missing. Set PIPEDREAM_API_ORIGIN or pass --api-origin.",
        ),
    ];
    for (setting, message) in cases {
        let error = Error::MissingSettings {
            settings: vec![setting],
        };
        assert_eq!(describe(&error), usage(message));
    }
}

#[test]
fn several_missing_settings_share_one_message() {
    let settings = vec![
        Setting::ClientId,
        Setting::ClientSecret,
        Setting::ProjectId,
        Setting::Environment,
    ];
    assert_eq!(
        describe(&Error::MissingSettings { settings }),
        usage(
            "The client ID, client secret, project ID, and environment are missing. Set \
             PIPEDREAM_CLIENT_ID, PIPEDREAM_CLIENT_SECRET, PIPEDREAM_PROJECT_ID, and \
             PIPEDREAM_ENVIRONMENT, or pass their flags."
        )
    );
    let settings = vec![Setting::ClientSecret, Setting::Environment];
    assert_eq!(
        describe(&Error::MissingSettings { settings }),
        usage(
            "The client secret and environment are missing. Set PIPEDREAM_CLIENT_SECRET and \
             PIPEDREAM_ENVIRONMENT, or pass their flags."
        )
    );
    assert_eq!(
        describe(&Error::MissingSettings {
            settings: Vec::new()
        })
        .exit,
        Exit::Usage
    );
}

#[test]
fn invalid_settings_name_their_source() {
    let cases = [
        (
            Setting::ProjectId,
            Supply::Variable,
            "PIPEDREAM_PROJECT_ID is not a valid project ID. Use the project ID that starts \
             with proj_.",
        ),
        (
            Setting::Environment,
            Supply::Variable,
            "PIPEDREAM_ENVIRONMENT is not a valid environment. Use development or production.",
        ),
        (
            Setting::ApiOrigin,
            Supply::Flag,
            "--api-origin is not a valid API origin. Use an HTTPS origin without a path, \
             such as https://api.pipedream.com.",
        ),
        (
            Setting::ClientId,
            Supply::Flag,
            "--client-id is not a valid client ID. Copy it again from your Pipedream OAuth \
             client.",
        ),
    ];
    for (setting, supply, message) in cases {
        assert_eq!(
            describe(&Error::InvalidSetting { setting, supply }),
            usage(message)
        );
    }
}

#[test]
fn credential_and_variable_failures_name_their_sources() {
    assert_eq!(
        describe(&Error::InvalidCredentials {
            client_id: Supply::Variable,
            client_secret: Supply::Flag,
        }),
        usage(
            "The client credentials from PIPEDREAM_CLIENT_ID and --client-secret are not \
             valid. Copy both values again from your Pipedream OAuth client."
        )
    );
    assert_eq!(
        describe(&Error::UnreadableVariable {
            setting: Setting::ClientSecret
        }),
        usage("PIPEDREAM_CLIENT_SECRET holds characters that the CLI cannot read. Set it again.")
    );
}

#[test]
fn invalid_arguments_name_the_argument_and_its_rule() {
    let identifier = |kind| pipedream_connect_types::Error::InvalidIdentifier { kind };
    let cases = [
        (
            argument(Argument::Category, identifier(IdentifierKind::AppCategory)),
            "--category is not a valid category ID. Use an ID that starts with appcat_.",
        ),
        (
            argument(Argument::Action, identifier(IdentifierKind::ComponentKey)),
            "<ACTION> is not a valid component key. Use 1 to 250 visible ASCII characters \
             without /, \\, ?, or #.",
        ),
        (
            argument(Argument::After, identifier(IdentifierKind::Cursor)),
            "--after is not a valid cursor. Copy the cursor from the page_info of an earlier \
             result.",
        ),
        (
            argument(
                Argument::Limit,
                pipedream_connect_types::Error::OutOfRange {
                    field: ValidationField::Limit,
                    minimum: 1,
                    maximum: 200,
                },
            ),
            "--limit must be from 1 to 200.",
        ),
        (
            argument(
                Argument::Props,
                pipedream_connect_types::Error::InvalidHttpsUrl {
                    field: UrlField::WebhookUri,
                },
            ),
            "--props must be an HTTPS URL.",
        ),
        (
            argument(
                Argument::After,
                pipedream_connect_types::Error::ConflictingCursors,
            ),
            "Pass --after or --before, not both.",
        ),
        (
            argument(Argument::App, pipedream_connect_types::Error::StuckCursor),
            "<APP> is not valid. Check the value, then run the command again.",
        ),
    ];
    for (failure, message) in cases {
        assert_eq!(failure, usage(message));
    }
}

#[test]
fn json_failures_give_the_position_and_the_expected_shape() {
    let failure = |problem| {
        describe(&Error::InvalidJson {
            argument: Argument::Props,
            line: 3,
            column: 7,
            problem,
        })
    };
    assert_eq!(
        failure(JsonProblem::Syntax),
        usage(
            "--props is not valid JSON at line 3, column 7. Fix the JSON, then run the \
             command again."
        )
    );
    assert_eq!(
        failure(JsonProblem::Shape),
        usage(
            "--props must be a JSON object that maps prop names to values. Check line 3, \
             column 7."
        )
    );
}

#[test]
fn unreadable_inputs_give_a_recovery_for_the_failure_kind() {
    let failure = |kind| {
        describe(&Error::UnreadableInput {
            argument: Argument::Props,
            cause: io::Error::from(kind),
        })
    };
    let cases = [
        (
            io::ErrorKind::NotFound,
            "The --props file does not exist. Check the path.",
        ),
        (
            io::ErrorKind::PermissionDenied,
            "The CLI cannot open the --props file. Check its permissions.",
        ),
        (
            io::ErrorKind::IsADirectory,
            "The --props path is a directory. Pass a file.",
        ),
        (
            io::ErrorKind::UnexpectedEof,
            "The CLI cannot read --props. Check the input, then run the command again.",
        ),
    ];
    for (kind, message) in cases {
        assert_eq!(failure(kind), usage(message));
    }
}
