//! Tests that each identifier hint states the whole rule of its identifier.

use pipedream_connect_types::{
    AccountId, AppCategoryId, DeployedTriggerId, IdentifierKind, OauthAppId, ProjectId,
};

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
        "Use an ID that starts with proj_ and then has 1 to 245 ASCII letters or digits.",
    ),
    (
        IdentifierKind::Account,
        "account ID",
        "Use an ID that starts with apn_ and then has 1 to 246 ASCII letters or digits.",
    ),
    (
        IdentifierKind::DeployedTrigger,
        "deployed trigger ID",
        "Use an ID that starts with dc_, hi_, or ti_ and then has 1 to 247 ASCII letters or \
         digits.",
    ),
    (
        IdentifierKind::OauthApp,
        "OAuth app ID",
        "Use an ID that starts with oa_ and then has 1 to 247 ASCII letters or digits.",
    ),
    (
        IdentifierKind::AppCategory,
        "category ID",
        "Use an ID that starts with appcat_ and then has 1 to 243 ASCII letters or digits.",
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

/// Whether one SDK identifier rule accepts a value.
type Accepts = fn(String) -> bool;

#[test]
fn prefixed_hints_give_the_sdk_length_limit() {
    let cases: [(&str, usize, Accepts); 7] = [
        ("proj_", 245, |value| ProjectId::new(value).is_ok()),
        ("apn_", 246, |value| AccountId::new(value).is_ok()),
        ("dc_", 247, |value| DeployedTriggerId::new(value).is_ok()),
        ("hi_", 247, |value| DeployedTriggerId::new(value).is_ok()),
        ("ti_", 247, |value| DeployedTriggerId::new(value).is_ok()),
        ("oa_", 247, |value| OauthAppId::new(value).is_ok()),
        ("appcat_", 243, |value| AppCategoryId::new(value).is_ok()),
    ];
    for (prefix, longest, accepts) in cases {
        let id = |length: usize| format!("{prefix}{}", "a".repeat(length));
        assert!(accepts(id(1)) && accepts(id(longest)), "{prefix}");
        assert!(!accepts(id(0)) && !accepts(id(longest + 1)), "{prefix}");
        assert!(!accepts(format!("{prefix}\u{e9}")), "{prefix}");
        let (.., hint) = HINTS
            .iter()
            .find(|(.., hint)| hint.contains(prefix))
            .unwrap();
        assert!(
            hint.ends_with(&format!("then has 1 to {longest} ASCII letters or digits.")),
            "{prefix}: {hint}"
        );
    }
}
