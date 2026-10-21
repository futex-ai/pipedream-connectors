//! Tests for the exit status and the text of each SDK failure.

use std::io;
use std::time::Duration;

use pipedream_connect::error::{Error, InvalidRequest};
use pipedream_connect_types::ExecutionError;
use serde_json::json;

use crate::report::api::{client, describe};
use crate::report::exit::Exit;
use crate::report::failure::Failure;
use crate::target::Target;

const PROVIDER_TEXT: &str = "provider text with token xoxb-hidden";

fn execution(error: serde_json::Value) -> Error {
    let error: ExecutionError = serde_json::from_value(error).unwrap();
    Error::ExecutionFailed {
        status: 400,
        error: Box::new(error),
    }
}

#[test]
fn authentication_failures_exit_three_without_provider_text() {
    let unauthorized = describe(
        Target::Apps,
        &Error::Unauthorized {
            message: Some(PROVIDER_TEXT.to_owned()),
        },
    );
    assert_eq!(
        unauthorized,
        Failure::new(
            Exit::Authentication,
            "Pipedream rejected the client credentials. Check the client ID and the client \
             secret."
        )
    );
    let forbidden = describe(
        Target::Apps,
        &Error::Forbidden {
            message: Some(PROVIDER_TEXT.to_owned()),
        },
    );
    assert_eq!(forbidden.exit, Exit::Authentication);
    assert!(!forbidden.message.contains(PROVIDER_TEXT));
}

#[test]
fn not_found_names_the_identifier_to_check() {
    let cases = [
        (
            Target::App,
            "Pipedream did not find the app. Check the app slug or ID.",
        ),
        (
            Target::Component,
            "Pipedream did not find the component. Check the component key and the version.",
        ),
        (
            Target::ActionRun,
            "Pipedream did not find the action. Check the action key and the version.",
        ),
        (
            Target::Components,
            "Pipedream did not find the listed resource. Check the project ID and the filters.",
        ),
    ];
    for (target, message) in cases {
        let failure = describe(
            target,
            &Error::NotFound {
                message: Some(PROVIDER_TEXT.to_owned()),
            },
        );
        assert_eq!(failure, Failure::new(Exit::NotFound, message));
    }
}

#[test]
fn rate_limits_exit_five_with_the_wait_time() {
    let limited = |retry_after| Error::RateLimited {
        retry_after,
        limit: Some(100),
        remaining: Some(0),
        reset: None,
    };
    let cases = [
        (
            Some(Duration::from_secs(30)),
            "Pipedream rate limited the request. Wait 30 seconds, then run the command again.",
        ),
        (
            Some(Duration::from_millis(500)),
            "Pipedream rate limited the request. Wait 1 second, then run the command again.",
        ),
        (
            None,
            "Pipedream rate limited the request. Wait a moment, then run the command again.",
        ),
    ];
    for (retry_after, message) in cases {
        assert_eq!(
            describe(Target::Categories, &limited(retry_after)),
            Failure::new(Exit::RateLimited, message)
        );
    }
}

#[test]
fn unavailability_warns_when_a_mutation_may_have_run() {
    assert_eq!(
        describe(Target::Apps, &Error::Unavailable { status: Some(503) }),
        Failure::new(
            Exit::Unavailable,
            "Pipedream is unavailable with status 503. Try again later."
        )
    );
    assert_eq!(
        describe(Target::ActionRun, &Error::Unavailable { status: None }),
        Failure::new(
            Exit::Unavailable,
            "Pipedream did not respond. The action may have run. Check its effects before you \
             run it again."
        )
    );
}

#[test]
fn other_api_failures_exit_seven_without_provider_text() {
    let rejected = describe(
        Target::Actions,
        &Error::Rejected {
            status: 422,
            code: Some("invalid_input".to_owned()),
            message: Some(PROVIDER_TEXT.to_owned()),
        },
    );
    assert_eq!(
        rejected,
        Failure::new(
            Exit::Rejected,
            "Pipedream rejected the request with status 422. Check the command arguments."
        )
    );
    assert_eq!(describe(Target::App, &Error::Accepted).exit, Exit::Rejected);
    assert_eq!(
        describe(Target::Apps, &Error::ResponseTooLarge { limit: 10 << 20 }),
        Failure::new(
            Exit::Rejected,
            "The Pipedream response is larger than 10 MiB. Request fewer records with --limit."
        )
    );
    assert_eq!(
        describe(Target::App, &Error::ResponseTooLarge { limit: 1000 }).message,
        "The Pipedream response is larger than the 1000 bytes limit of the CLI."
    );
}

#[test]
fn execution_failures_name_the_failure_origin() {
    let cases = [
        (
            json!({"attribution": {"origin": "upstream_api", "last_call": {"status": 401}}}),
            "The action failed at the app API with status 401.",
        ),
        (
            json!({"attribution": {"origin": "upstream_api"}}),
            "The action failed at the app API.",
        ),
        (
            json!({"attribution": {"origin": "component_code"}}),
            "The action failed in its component code.",
        ),
        (
            json!({"attribution": {"origin": "network_io"}}),
            "The action failed on a network call.",
        ),
        (
            json!({"attribution": {"origin": "response_parsing"}}),
            "The action failed while it read an app response.",
        ),
        (
            json!({"attribution": {"origin": "future_origin"}}),
            "The action failed.",
        ),
        (json!({"name": "Error"}), "The action failed."),
    ];
    for (mut error, summary) in cases {
        error["message"] = json!(PROVIDER_TEXT);
        let failure = describe(Target::ActionRun, &execution(error));
        assert_eq!(
            failure,
            Failure::new(
                Exit::Rejected,
                format!("{summary} Check the configured props, then run the command again.")
            )
        );
    }
}

#[test]
fn invalid_requests_exit_two_and_internal_failures_exit_one() {
    let cases = [
        (
            InvalidRequest::Header,
            "A request header is not valid. Check the header names and values.",
        ),
        (
            InvalidRequest::Model(pipedream_connect_types::Error::ConflictingCursors),
            "A request value is not valid. Check the command arguments.",
        ),
        (
            InvalidRequest::Origin,
            "The connection settings are not valid. Check the configuration flags and variables.",
        ),
    ];
    for (reason, message) in cases {
        assert_eq!(
            describe(Target::Apps, &Error::InvalidRequest(reason)),
            Failure::new(Exit::Usage, message)
        );
    }
    let internal = describe(Target::App, &Error::internal(io::Error::other("decode")));
    assert_eq!(internal.exit, Exit::Internal);
    assert!(
        internal
            .message
            .starts_with("Pipedream returned a response")
    );
}

#[test]
fn client_construction_failures_separate_settings_from_internal_errors() {
    assert_eq!(
        client(&Error::InvalidRequest(InvalidRequest::Credentials)).exit,
        Exit::Usage
    );
    assert_eq!(
        client(&Error::internal(io::Error::other("tls"))),
        Failure::new(
            Exit::Internal,
            "The CLI could not start its HTTP client. Run the command again."
        )
    );
}
