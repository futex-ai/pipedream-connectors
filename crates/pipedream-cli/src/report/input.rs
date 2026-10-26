//! The text of configuration and input failures. No request was sent for these.

use std::io::ErrorKind;

use pipedream_connect_types::IdentifierKind;

use crate::argument::Argument;
use crate::error::{HeaderProblem, JsonProblem};
use crate::inputs::MAX_DEPTH;
use crate::settings::{Setting, Supply};

fn noun(setting: Setting) -> &'static str {
    match setting {
        Setting::ClientId => "client ID",
        Setting::ClientSecret => "client secret",
        Setting::ProjectId => "project ID",
        Setting::Environment => "environment",
        Setting::ApiOrigin => "API origin",
    }
}

/// The text for required settings that have neither a flag nor a variable.
pub(super) fn missing(settings: &[Setting]) -> String {
    match settings {
        [] => {
            "A required setting is missing. Check the configuration flags and variables.".to_owned()
        }
        [setting] => missing_one(*setting),
        settings => {
            let nouns: Vec<&str> = settings.iter().map(|setting| noun(*setting)).collect();
            let variables: Vec<&str> = settings.iter().map(|setting| setting.variable()).collect();
            format!(
                "The {} are missing. Set {}.",
                join(&nouns),
                join(&variables)
            )
        }
    }
}

/// Joins words as a list, such as `a, b, and c`.
fn join(words: &[&str]) -> String {
    match words {
        [] => String::new(),
        [word] => (*word).to_owned(),
        [first, second] => format!("{first} and {second}"),
        [rest @ .., last] => format!("{}, and {last}", rest.join(", ")),
    }
}

fn missing_one(setting: Setting) -> String {
    let (variable, flag, noun) = (setting.variable(), setting.flag(), noun(setting));
    match setting {
        Setting::ClientSecret => format!("The {noun} is missing. Set {variable}."),
        Setting::Environment => format!(
            "The {noun} is missing. Set {variable} to development or production, or pass {flag}."
        ),
        Setting::ClientId | Setting::ProjectId | Setting::ApiOrigin => {
            format!("The {noun} is missing. Set {variable} or pass {flag}.")
        }
    }
}

/// The text for a setting value that fails its grammar, naming its source.
pub(super) fn invalid_setting(setting: Setting, supply: Supply) -> String {
    let name = supply.name(setting);
    match setting {
        Setting::ProjectId => {
            format!(
                "{name} is not a valid project ID. Use the project ID that starts with proj_ and \
                 then has only letters and digits."
            )
        }
        Setting::Environment => {
            format!("{name} is not a valid environment. Use development or production.")
        }
        Setting::ApiOrigin => format!(
            "{name} is not a valid API origin. \
             Use an HTTPS origin that has only a host and an optional port, such as \
             https://api.pipedream.com."
        ),
        Setting::ClientId | Setting::ClientSecret => format!(
            "{name} is not a valid {}. Copy it again from your Pipedream OAuth client.",
            noun(setting)
        ),
    }
}

/// The text for an argument that fails its model rule.
pub(super) fn invalid_argument(
    argument: Argument,
    cause: &pipedream_connect_types::Error,
) -> String {
    match cause {
        pipedream_connect_types::Error::InvalidIdentifier { kind } => format!(
            "{argument} is not a valid {}. {}",
            identifier(*kind),
            identifier_hint(*kind)
        ),
        pipedream_connect_types::Error::OutOfRange {
            minimum, maximum, ..
        } => format!("{argument} must be from {minimum} to {maximum}."),
        pipedream_connect_types::Error::InvalidHttpsUrl { .. } => {
            format!("{argument} must be an HTTPS URL with a host.")
        }
        pipedream_connect_types::Error::ConflictingCursors => {
            "Pass --after or --before, not both.".to_owned()
        }
        pipedream_connect_types::Error::StuckCursor
        | pipedream_connect_types::Error::InvalidTimestamp
        | pipedream_connect_types::Error::Internal(_) => {
            format!("{argument} is not valid. Check the value, then run the command again.")
        }
    }
}

fn identifier(kind: IdentifierKind) -> &'static str {
    match kind {
        IdentifierKind::Project => "project ID",
        IdentifierKind::ExternalUser => "external user ID",
        IdentifierKind::Account => "account ID",
        IdentifierKind::DeployedTrigger => "deployed trigger ID",
        IdentifierKind::OauthApp => "OAuth app ID",
        IdentifierKind::AppCategory => "category ID",
        IdentifierKind::AppOverride => "app override ID",
        IdentifierKind::AppSlug => "app slug",
        IdentifierKind::ComponentKey => "component key",
        IdentifierKind::PropName => "prop name",
        IdentifierKind::Webhook => "webhook ID",
        IdentifierKind::ConnectToken => "Connect token",
        IdentifierKind::Cursor => "cursor",
        IdentifierKind::WorkflowEndpoint => "workflow ID",
    }
}

/// States the whole rule of an identifier, so the hint never names a rule that the input meets.
fn identifier_hint(kind: IdentifierKind) -> String {
    let prefixed = |prefix: &str| {
        format!("Use an ID that starts with {prefix} and then has only letters and digits.")
    };
    match kind {
        IdentifierKind::Project => prefixed("proj_"),
        IdentifierKind::Account => prefixed("apn_"),
        IdentifierKind::DeployedTrigger => prefixed("dc_, hi_, or ti_"),
        IdentifierKind::OauthApp => prefixed("oa_"),
        IdentifierKind::AppCategory => prefixed("appcat_"),
        IdentifierKind::ConnectToken => {
            "Use a token that starts with ctok_ and then has 32 lowercase hexadecimal digits."
                .to_owned()
        }
        IdentifierKind::Cursor => "A cursor has 1 to 2048 visible ASCII characters without \
                                   spaces. Copy it from the page_info of an earlier result."
            .to_owned(),
        IdentifierKind::ExternalUser
        | IdentifierKind::AppOverride
        | IdentifierKind::AppSlug
        | IdentifierKind::ComponentKey
        | IdentifierKind::PropName
        | IdentifierKind::Webhook
        | IdentifierKind::WorkflowEndpoint => "Use 1 to 250 visible ASCII characters without \
                                               spaces, /, \\, ?, or #. The values . and .. \
                                               are not valid."
            .to_owned(),
    }
}

/// The text for an input that cannot be read, by the kind of failure.
pub(super) fn unreadable(argument: Argument, kind: ErrorKind) -> String {
    match kind {
        ErrorKind::NotFound => format!("The {argument} file does not exist. Check the path."),
        ErrorKind::PermissionDenied => {
            format!("The CLI cannot open the {argument} file. Check its permissions.")
        }
        ErrorKind::IsADirectory => format!("The {argument} path is a directory. Pass a file."),
        _ => {
            format!("The CLI cannot read {argument}. Check the input, then run the command again.")
        }
    }
}

/// The text for a variable that holds bytes that are not UTF-8.
pub(super) fn unreadable_variable(setting: Setting) -> String {
    format!(
        "{} holds characters that the CLI cannot read. Set it again.",
        setting.variable()
    )
}

/// The text for credentials that the SDK rejects, naming both sources.
pub(super) fn invalid_credentials(client_id: Supply, client_secret: Supply) -> String {
    format!(
        "The client credentials from {} and {} are not valid. \
         Copy both values again from your Pipedream OAuth client.",
        client_id.name(Setting::ClientId),
        client_secret.name(Setting::ClientSecret)
    )
}

/// The text for a `--header` value that the proxy cannot forward. It never shows the value.
pub(super) fn invalid_header(position: usize, problem: &HeaderProblem) -> String {
    match problem {
        HeaderProblem::Format => {
            format!("--header number {position} must look like 'Name: value'.")
        }
        HeaderProblem::Name => format!(
            "--header number {position} has a name that HTTP does not allow. \
             Use letters, digits, and hyphens."
        ),
        HeaderProblem::Value { name } => format!(
            "The {name} header has a value that HTTP does not allow. Remove its control characters."
        ),
        HeaderProblem::Reserved { name } => format!(
            "Pipedream reserves the {name} header, so the proxy does not forward it. Remove it."
        ),
    }
}

/// The text for a JSON input that did not decode, with only its position.
pub(super) fn invalid_json(
    argument: Argument,
    line: usize,
    column: usize,
    problem: JsonProblem,
) -> String {
    match problem {
        JsonProblem::Syntax => format!(
            "{argument} is not valid JSON at line {line}, column {column}. \
             Fix the JSON, then run the command again."
        ),
        JsonProblem::Shape => format!(
            "{argument} must be a JSON object that maps prop names to values. \
             Check line {line}, column {column}."
        ),
        JsonProblem::Depth => format!(
            "{argument} nests JSON too deeply at line {line}, column {column}. \
             Use at most {MAX_DEPTH} levels of arrays and objects."
        ),
    }
}

#[cfg(test)]
#[path = "_tests_/input/mod.rs"]
mod input_tests;
