//! Tests for the text of missing, invalid, and unreadable settings.

use crate::error::Error;
use crate::report::exit::Exit;
use crate::report::failure::describe;
use crate::settings::DEFAULT_ORIGIN;
use crate::settings::{Setting, Supply};

use super::usage;

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
        assert_eq!(describe(&error, DEFAULT_ORIGIN), usage(message));
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
        describe(&Error::MissingSettings { settings }, DEFAULT_ORIGIN),
        usage(
            "The client ID, client secret, project ID, and environment are missing. Set \
             PIPEDREAM_CLIENT_ID, PIPEDREAM_CLIENT_SECRET, PIPEDREAM_PROJECT_ID, and \
             PIPEDREAM_ENVIRONMENT."
        )
    );
    let settings = vec![Setting::ClientSecret, Setting::Environment];
    assert_eq!(
        describe(&Error::MissingSettings { settings }, DEFAULT_ORIGIN),
        usage(
            "The client secret and environment are missing. Set PIPEDREAM_CLIENT_SECRET and \
             PIPEDREAM_ENVIRONMENT."
        )
    );
    assert_eq!(
        describe(
            &Error::MissingSettings {
                settings: Vec::new()
            },
            DEFAULT_ORIGIN
        )
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
             with proj_ and then has 1 to 245 ASCII letters or digits.",
        ),
        (
            Setting::Environment,
            Supply::Variable,
            "PIPEDREAM_ENVIRONMENT is not a valid environment. Use development or production.",
        ),
        (
            Setting::ApiOrigin,
            Supply::Flag,
            "--api-origin is not a valid API origin. Use an HTTPS origin that has only a host \
             and an optional port, without surrounding spaces, such as \
             https://api.pipedream.com.",
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
            describe(&Error::InvalidSetting { setting, supply }, DEFAULT_ORIGIN),
            usage(message)
        );
    }
}

#[test]
fn credential_and_variable_failures_name_their_sources() {
    assert_eq!(
        describe(
            &Error::InvalidCredentials {
                client_id: Supply::Variable,
                client_secret: Supply::Flag,
            },
            DEFAULT_ORIGIN
        ),
        usage(
            "The client credentials from PIPEDREAM_CLIENT_ID and --client-secret are not \
             valid. Copy both values again from your Pipedream OAuth client."
        )
    );
    assert_eq!(
        describe(
            &Error::UnreadableVariable {
                setting: Setting::ClientSecret
            },
            DEFAULT_ORIGIN
        ),
        usage("PIPEDREAM_CLIENT_SECRET holds characters that the CLI cannot read. Set it again.")
    );
}
