//! The failure text that depends on the resource of a command.
//!
//! A new command target adds one arm here, and every API failure message follows.

use crate::target::Target;

/// What failure messages say about one target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TargetText {
    /// What Pipedream did not find, for a 404.
    pub(super) missing: &'static str,
    /// What the user checks after a 404.
    pub(super) missing_action: &'static str,
    /// The subject of an execution failure, such as `The action`.
    pub(super) subject: &'static str,
    /// What the user does after an execution failure.
    pub(super) execution_action: &'static str,
    /// The warning for a change that may have happened without a readable answer.
    pub(super) uncertain: Option<&'static str>,
    /// Whether `--limit` can make the response smaller.
    pub(super) paged: bool,
}

/// Text for a command that only reads.
const fn read(missing: &'static str, missing_action: &'static str, paged: bool) -> TargetText {
    TargetText {
        missing,
        missing_action,
        subject: "The request",
        execution_action: "Check the command arguments, then run the command again.",
        uncertain: None,
        paged,
    }
}

/// The 404 text of every project-scoped list command.
const LISTED: &str = "Pipedream did not find the listed resource.";

/// The 404 action of every project-scoped list command.
const LISTED_ACTION: &str = "Check the project ID and the filters.";

/// Returns the failure text for `target`.
pub(super) fn text(target: Target) -> TargetText {
    match target {
        Target::Apps => read(
            "Pipedream did not find the apps.",
            "Check the API origin and the filters.",
            true,
        ),
        Target::Categories => read(
            "Pipedream did not find the app categories.",
            "Check the API origin.",
            false,
        ),
        Target::Components | Target::Actions => read(LISTED, LISTED_ACTION, true),
        Target::App => read(
            "Pipedream did not find the app.",
            "Check the app slug or ID.",
            false,
        ),
        Target::Component => read(
            "Pipedream did not find the component.",
            "Check the component key and the version.",
            false,
        ),
        Target::ActionRun => TargetText {
            missing: "Pipedream did not find the action.",
            missing_action: "Check the action key and the version.",
            subject: "The action",
            execution_action: "Check the configured props, then run the command again.",
            uncertain: Some("The action may have run. Check its effects before you run it again."),
            paged: false,
        },
    }
}
