//! The failure text that depends on the resource of a command.
//!
//! A new command target adds one arm here, and every API failure message follows.

use crate::target::Target;

/// What failure messages say about one target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TargetText {
    /// The message when Pipedream does not find the resource.
    pub(super) not_found: &'static str,
    /// The subject of an execution failure, such as `The action`.
    pub(super) subject: &'static str,
    /// The warning for a change that got no answer. A read has none.
    pub(super) unanswered: Option<&'static str>,
    /// Whether `--limit` can make the response smaller.
    pub(super) paged: bool,
}

/// The not-found message of every list command.
const LISTED: &str =
    "Pipedream did not find the listed resource. Check the project ID and the filters.";

/// Text for a command that only reads.
const fn read(not_found: &'static str, paged: bool) -> TargetText {
    TargetText {
        not_found,
        subject: "The request",
        unanswered: None,
        paged,
    }
}

/// Returns the failure text for `target`.
pub(super) fn text(target: Target) -> TargetText {
    match target {
        Target::Apps | Target::Components | Target::Actions => read(LISTED, true),
        Target::Categories => read(LISTED, false),
        Target::App => read(
            "Pipedream did not find the app. Check the app slug or ID.",
            false,
        ),
        Target::Component => read(
            "Pipedream did not find the component. Check the component key and the version.",
            false,
        ),
        Target::ActionRun => TargetText {
            not_found: "Pipedream did not find the action. Check the action key and the version.",
            subject: "The action",
            unanswered: Some("The action may have run. Check its effects before you run it again."),
            paged: false,
        },
    }
}
