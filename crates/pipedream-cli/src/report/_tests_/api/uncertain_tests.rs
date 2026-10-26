//! Tests for failures where Pipedream may have changed data without a readable answer.

use std::io;

use pipedream_connect::error::Error;

use crate::report::api::describe;
use crate::report::exit::Exit;
use crate::report::failure::Failure;
use crate::target::Target;

use super::{MAY_HAVE_RUN, REPORT, text};

#[test]
fn unavailability_names_the_origin_and_warns_after_a_mutation() {
    assert_eq!(
        text(
            Target::Apps,
            &Error::Unavailable {
                status: None,
                retry_after: None
            }
        ),
        Failure::new(
            Exit::Unavailable,
            "The CLI could not reach Pipedream at https://api.pipedream.com. Check the network \
             and the API origin, then try again."
        )
    );
    assert_eq!(
        describe(
            Target::Apps,
            &Error::Unavailable {
                status: Some(503),
                retry_after: None
            },
            "https://api.example.com"
        ),
        Failure::new(
            Exit::Unavailable,
            "Pipedream at https://api.example.com is unavailable with status 503. Try again later."
        )
    );
    assert_eq!(
        text(
            Target::ActionRun,
            &Error::Unavailable {
                status: None,
                retry_after: None
            }
        ),
        Failure::new(
            Exit::Unavailable,
            format!(
                "The CLI could not reach Pipedream at https://api.pipedream.com. {MAY_HAVE_RUN}"
            )
        )
    );
}

#[test]
fn unreadable_responses_exit_one_and_warn_after_a_mutation() {
    let large = Error::ResponseTooLarge {
        status: 0,
        limit: 10 << 20,
    };
    assert_eq!(
        text(Target::Apps, &large),
        Failure::new(
            Exit::Internal,
            "The Pipedream response is larger than 10 MiB, so the CLI could not read it. \
             Request fewer records with --limit."
        )
    );
    assert_eq!(
        text(
            Target::App,
            &Error::ResponseTooLarge {
                status: 0,
                limit: 1000
            }
        ),
        Failure::new(
            Exit::Internal,
            format!(
                "The Pipedream response is larger than 1000 bytes, so the CLI could not read \
                 it. {REPORT}"
            )
        )
    );
    assert_eq!(
        text(Target::ActionRun, &large),
        Failure::new(
            Exit::Internal,
            format!(
                "The Pipedream response is larger than 10 MiB, so the CLI could not read it. \
                 {MAY_HAVE_RUN}"
            )
        )
    );
    let decode = || Error::internal(io::Error::other("decode"));
    let unreadable = "Pipedream returned a response that the CLI cannot read.";
    assert_eq!(
        text(Target::Component, &decode()),
        Failure::new(Exit::Internal, format!("{unreadable} {REPORT}"))
    );
    assert_eq!(
        text(Target::ActionRun, &decode()),
        Failure::new(Exit::Internal, format!("{unreadable} {MAY_HAVE_RUN}"))
    );
}
