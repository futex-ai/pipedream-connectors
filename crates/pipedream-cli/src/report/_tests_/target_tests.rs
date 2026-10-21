//! Tests that the failure text of each target fits what its command does.

use crate::report::target::text;
use crate::target::Target;

/// Every target. `listed` fails to compile when a new target is missing here.
const ALL: [Target; 21] = [
    Target::Apps,
    Target::App,
    Target::Categories,
    Target::Components,
    Target::Component,
    Target::Actions,
    Target::ActionRun,
    Target::TokenCreate,
    Target::Accounts,
    Target::Account,
    Target::Users,
    Target::ProxyRead,
    Target::ProxyWrite,
    Target::Triggers,
    Target::Trigger,
    Target::TriggerDeploy,
    Target::TriggerDelete,
    Target::TriggerEvents,
    Target::ProjectInfo,
    Target::Usage,
    Target::Webhook,
];

/// Names every variant, so a new target breaks the build until `ALL` lists it.
fn listed(target: Target) -> bool {
    match target {
        Target::Apps
        | Target::App
        | Target::Categories
        | Target::Components
        | Target::Component
        | Target::Actions
        | Target::ActionRun
        | Target::TokenCreate
        | Target::Accounts
        | Target::Account
        | Target::Users
        | Target::ProxyRead
        | Target::ProxyWrite
        | Target::Triggers
        | Target::Trigger
        | Target::TriggerDeploy
        | Target::TriggerDelete
        | Target::TriggerEvents
        | Target::ProjectInfo
        | Target::Usage
        | Target::Webhook => ALL.contains(&target),
    }
}

#[test]
fn every_target_is_listed() {
    assert!(ALL.iter().all(|target| listed(*target)));
}

#[test]
fn only_commands_that_change_data_warn_that_the_change_may_have_happened() {
    let changes = [
        Target::ActionRun,
        Target::TokenCreate,
        Target::ProxyWrite,
        Target::TriggerDeploy,
        Target::TriggerDelete,
    ];
    for target in ALL {
        assert_eq!(
            text(target).uncertain.is_some(),
            changes.contains(&target),
            "{target:?}"
        );
    }
}

#[test]
fn only_commands_with_a_limit_flag_suggest_it() {
    let limited = [
        Target::Apps,
        Target::Components,
        Target::Actions,
        Target::Accounts,
        Target::Users,
        Target::Triggers,
        Target::TriggerEvents,
    ];
    for target in ALL {
        assert_eq!(text(target).paged, limited.contains(&target), "{target:?}");
    }
}

#[test]
fn reads_never_mention_configured_props() {
    for target in ALL {
        let target_text = text(target);
        if target_text.uncertain.is_none() {
            assert!(
                !target_text.execution_action.contains("props"),
                "{target:?}"
            );
        }
    }
}

#[test]
fn commands_outside_a_project_never_suggest_the_project_id() {
    for target in [Target::Apps, Target::Categories, Target::Usage] {
        assert!(
            !text(target).missing_action.contains("project"),
            "{target:?}"
        );
    }
}
