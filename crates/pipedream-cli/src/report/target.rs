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

/// Text for a command that changes data.
const fn change(
    missing: (&'static str, &'static str),
    subject: &'static str,
    execution_action: &'static str,
    uncertain: &'static str,
) -> TargetText {
    TargetText {
        missing: missing.0,
        missing_action: missing.1,
        subject,
        execution_action,
        uncertain: Some(uncertain),
        paged: false,
    }
}

/// The 404 text of every project-scoped list command.
const LISTED: &str = "Pipedream did not find the listed resource.";

/// The 404 action of every project-scoped list command.
const LISTED_ACTION: &str = "Check the project ID and the filters.";

/// The 404 text of a command that names one deployed trigger.
const DEPLOYED: (&str, &str) = (
    "Pipedream did not find the deployed trigger.",
    "Check the trigger ID and the external user ID.",
);

/// The 404 text of a proxy request.
const PROXIED: (&str, &str) = (
    "Pipedream did not find the account or the proxy target.",
    "Check the account ID, the external user ID, and the URL.",
);

/// The execution action of a proxy request.
const PROXY_ACTION: &str = "Check the account and the request, then run the command again.";

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
        Target::Components | Target::Actions | Target::Accounts | Target::Users => {
            read(LISTED, LISTED_ACTION, true)
        }
        Target::Triggers => read(
            LISTED,
            "Check the project ID and the external user ID.",
            true,
        ),
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
        Target::Account => read(
            "Pipedream did not find the account.",
            "Check the account ID and the external user ID.",
            false,
        ),
        Target::Trigger => read(DEPLOYED.0, DEPLOYED.1, false),
        Target::TriggerEvents => read(DEPLOYED.0, DEPLOYED.1, true),
        Target::ProjectInfo | Target::Webhook => read(
            "Pipedream did not find the project.",
            "Check the project ID and the environment.",
            false,
        ),
        Target::Usage => read(
            "Pipedream did not find the usage records.",
            "Check the API origin and the time window.",
            false,
        ),
        Target::ProxyRead => TargetText {
            subject: "The proxy request",
            execution_action: PROXY_ACTION,
            ..read(PROXIED.0, PROXIED.1, false)
        },
        Target::ActionRun => change(
            (
                "Pipedream did not find the action.",
                "Check the action key and the version.",
            ),
            "The action",
            "Check the configured props, then run the command again.",
            "The action may have run. Check its effects before you run it again.",
        ),
        Target::TokenCreate => change(
            (
                "Pipedream did not find the project.",
                "Check the project ID and the environment.",
            ),
            "The token request",
            "Check the command arguments, then run the command again.",
            "Pipedream may have created the token. Run the command again for a new one.",
        ),
        Target::ProxyWrite => change(
            PROXIED,
            "The proxy request",
            PROXY_ACTION,
            "The upstream request may have run. Check its effects before you send it again.",
        ),
        Target::TriggerDeploy => change(
            (
                "Pipedream did not find the trigger.",
                "Check the component key and the version.",
            ),
            "The deployment",
            "Check the configured props, then run the command again.",
            "Pipedream may have deployed the trigger. Check triggers list before you deploy \
             it again.",
        ),
        Target::TriggerDelete => change(
            DEPLOYED,
            "The deletion",
            "Check the command arguments, then run the command again.",
            "Pipedream may have deleted the trigger. Check it with triggers get.",
        ),
    }
}

#[cfg(test)]
#[path = "_tests_/target_tests.rs"]
mod target_tests;
