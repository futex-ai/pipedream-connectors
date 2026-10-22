//! Tests for failures whose HTTP status decides the exit code.

use std::io;
use std::time::Duration;

use pipedream_connect::error::{Error, InvalidRequest};
use serde_json::json;

use crate::report::api::client;
use crate::report::exit::Exit;
use crate::report::failure::Failure;
use crate::target::Target;

use super::{PROVIDER_TEXT, execution, text};

#[test]
fn authentication_failures_exit_three_without_provider_text() {
    let message = Some(PROVIDER_TEXT.to_owned());
    assert_eq!(
        text(Target::Apps, &Error::Unauthorized { message }),
        Failure::new(
            Exit::Authentication,
            "Pipedream rejected the client credentials. Check the client ID and the client \
             secret."
        )
    );
    let message = Some(PROVIDER_TEXT.to_owned());
    let forbidden = text(Target::Apps, &Error::Forbidden { message });
    assert_eq!(forbidden.exit, Exit::Authentication);
    assert!(!forbidden.message.contains(PROVIDER_TEXT));
}

#[test]
fn not_found_names_what_to_check_for_each_target() {
    let cases = [
        (
            Target::Apps,
            "Pipedream did not find the apps. Check the API origin and the filters.",
        ),
        (
            Target::Categories,
            "Pipedream did not find the app categories. Check the API origin.",
        ),
        (
            Target::Components,
            "Pipedream did not find the listed resource. Check the project ID and the filters.",
        ),
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
    ];
    for (target, message) in cases {
        let error = Error::NotFound {
            message: Some(PROVIDER_TEXT.to_owned()),
        };
        assert_eq!(
            text(target, &error),
            Failure::new(Exit::NotFound, message),
            "{target:?}"
        );
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
        (Some(Duration::from_secs(30)), "Wait 30 seconds"),
        (Some(Duration::from_millis(500)), "Wait 1 second"),
        (None, "Wait a moment"),
    ];
    for (retry_after, wait) in cases {
        assert_eq!(
            text(Target::Categories, &limited(retry_after)),
            Failure::new(
                Exit::RateLimited,
                format!("Pipedream rate limited the request. {wait}, then run the command again.")
            )
        );
    }
}

#[test]
fn rejected_requests_exit_seven_without_provider_text() {
    let rejected = text(
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
    assert_eq!(text(Target::App, &Error::Accepted).exit, Exit::Rejected);
}

#[test]
fn execution_failures_name_the_origin_and_fit_the_command() {
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
        assert_eq!(
            text(Target::ActionRun, &execution(error)),
            Failure::new(
                Exit::Rejected,
                format!("{summary} Check the configured props, then run the command again.")
            )
        );
    }
    let read = text(Target::Components, &execution(json!({"name": "Error"})));
    assert_eq!(
        read.message,
        "The request failed. Check the command arguments, then run the command again."
    );
}

#[test]
fn invalid_requests_exit_two() {
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
        (
            InvalidRequest::ProxyTarget,
            "<URL> must be an HTTPS URL or a path that starts with /. Fix the proxy target.",
        ),
        (
            InvalidRequest::ProxyHeader,
            "Pipedream does not forward one of the --header values. Check the header names.",
        ),
        (
            InvalidRequest::WorkflowMethod,
            "Pipedream workflows accept GET, POST, PUT, PATCH, and DELETE. Use one of them.",
        ),
        (
            InvalidRequest::WorkflowEndpoint,
            "The workflow URL is not valid. Use an HTTPS URL without credentials or a fragment.",
        ),
    ];
    for (reason, message) in cases {
        assert_eq!(
            text(Target::Apps, &Error::InvalidRequest(reason)),
            Failure::new(Exit::Usage, message)
        );
    }
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
