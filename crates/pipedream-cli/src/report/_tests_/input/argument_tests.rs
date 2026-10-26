//! Tests for the text of invalid arguments and unreadable inputs.

use std::io;

use pipedream_connect_types::{IdentifierKind, UrlField, ValidationField};

use crate::argument::Argument;
use crate::error::{Error, JsonProblem};
use crate::report::failure::describe;
use crate::settings::DEFAULT_ORIGIN;

use super::{argument, usage};

#[test]
fn invalid_arguments_name_the_argument_and_its_rule() {
    let identifier = |kind| pipedream_connect_types::Error::InvalidIdentifier { kind };
    let cases = [
        (
            argument(Argument::Category, identifier(IdentifierKind::AppCategory)),
            "--category is not a valid category ID. Use an ID that starts with appcat_ and then \
             has only letters and digits.",
        ),
        (
            argument(Argument::Action, identifier(IdentifierKind::ComponentKey)),
            "<ACTION> is not a valid component key. Use 1 to 250 visible ASCII characters \
             without spaces, /, \\, ?, or #. The values . and .. are not valid.",
        ),
        (
            argument(Argument::After, identifier(IdentifierKind::Cursor)),
            "--after is not a valid cursor. A cursor has 1 to 2048 visible ASCII characters \
             without spaces. Copy it from the page_info of an earlier result.",
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
            "--props must be an HTTPS URL with a host.",
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
        describe(
            &Error::InvalidJson {
                argument: Argument::Props,
                line: 3,
                column: 7,
                problem,
            },
            DEFAULT_ORIGIN,
        )
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
    assert_eq!(
        failure(JsonProblem::Depth),
        usage(
            "--props nests JSON too deeply at line 3, column 7. Use at most 127 levels of \
             arrays and objects."
        )
    );
}

#[test]
fn counts_that_are_not_positive_name_their_flag() {
    assert_eq!(
        describe(
            &Error::InvalidCount {
                argument: Argument::MaxPages
            },
            DEFAULT_ORIGIN
        ),
        usage("--max-pages must be a whole number of 1 or more.")
    );
}

#[test]
fn unreadable_inputs_give_a_recovery_for_the_failure_kind() {
    let failure = |kind| {
        describe(
            &Error::UnreadableInput {
                argument: Argument::Props,
                cause: io::Error::from(kind),
            },
            DEFAULT_ORIGIN,
        )
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
