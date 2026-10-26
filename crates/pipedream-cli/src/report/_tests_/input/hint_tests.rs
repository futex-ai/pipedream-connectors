//! Tests that each identifier hint states the whole rule of its identifier.

use pipedream_connect_types::IdentifierKind;

use crate::argument::Argument;

use super::{argument, usage};

/// The rule of every identifier without a documented prefix.
const COMMON: &str = "Use 1 to 250 visible ASCII characters without spaces, /, \\, ?, or #. The \
                      values . and .. are not valid.";

/// Every identifier kind with its name and its whole rule.
const HINTS: [(IdentifierKind, &str, &str); 14] = [
    (
        IdentifierKind::Project,
        "project ID",
        "Use an ID that starts with proj_ and then has only letters and digits.",
    ),
    (
        IdentifierKind::Account,
        "account ID",
        "Use an ID that starts with apn_ and then has only letters and digits.",
    ),
    (
        IdentifierKind::DeployedTrigger,
        "deployed trigger ID",
        "Use an ID that starts with dc_, hi_, or ti_ and then has only letters and digits.",
    ),
    (
        IdentifierKind::OauthApp,
        "OAuth app ID",
        "Use an ID that starts with oa_ and then has only letters and digits.",
    ),
    (
        IdentifierKind::AppCategory,
        "category ID",
        "Use an ID that starts with appcat_ and then has only letters and digits.",
    ),
    (
        IdentifierKind::ConnectToken,
        "Connect token",
        "Use a token that starts with ctok_ and then has 32 lowercase hexadecimal digits.",
    ),
    (
        IdentifierKind::Cursor,
        "cursor",
        "A cursor has 1 to 2048 visible ASCII characters without spaces. Copy it from the \
         page_info of an earlier result.",
    ),
    (IdentifierKind::ExternalUser, "external user ID", COMMON),
    (IdentifierKind::AppOverride, "app override ID", COMMON),
    (IdentifierKind::AppSlug, "app slug", COMMON),
    (IdentifierKind::ComponentKey, "component key", COMMON),
    (IdentifierKind::PropName, "prop name", COMMON),
    (IdentifierKind::Webhook, "webhook ID", COMMON),
    (IdentifierKind::WorkflowEndpoint, "workflow ID", COMMON),
];

/// Names every kind, so a new identifier kind breaks the build until `HINTS` lists it.
fn listed(kind: IdentifierKind) -> bool {
    match kind {
        IdentifierKind::Project
        | IdentifierKind::Account
        | IdentifierKind::DeployedTrigger
        | IdentifierKind::OauthApp
        | IdentifierKind::AppCategory
        | IdentifierKind::ConnectToken
        | IdentifierKind::Cursor
        | IdentifierKind::ExternalUser
        | IdentifierKind::AppOverride
        | IdentifierKind::AppSlug
        | IdentifierKind::ComponentKey
        | IdentifierKind::PropName
        | IdentifierKind::Webhook
        | IdentifierKind::WorkflowEndpoint => HINTS.iter().any(|(listed, ..)| *listed == kind),
    }
}

#[test]
fn every_identifier_hint_states_the_whole_rule() {
    for (kind, name, hint) in HINTS {
        assert!(listed(kind), "{kind:?}");
        let cause = pipedream_connect_types::Error::InvalidIdentifier { kind };
        assert_eq!(
            argument(Argument::App, cause),
            usage(&format!("<APP> is not a valid {name}. {hint}")),
            "{kind:?}"
        );
    }
}
