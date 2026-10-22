//! The resources that commands read or change.

use std::fmt;

/// The resource that a command reads or changes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Target {
    /// The app catalog.
    Apps,
    /// One app.
    App,
    /// The app categories.
    Categories,
    /// The component registry.
    Components,
    /// One component.
    Component,
    /// The action registry.
    Actions,
    /// One action run.
    ActionRun,
    /// One Connect token creation.
    TokenCreate,
    /// The connected accounts.
    Accounts,
    /// One connected account.
    Account,
    /// The external users.
    Users,
    /// One proxy request that only reads.
    ProxyRead,
    /// One proxy request that can change upstream data.
    ProxyWrite,
    /// The deployed triggers of an external user.
    Triggers,
    /// One deployed trigger.
    Trigger,
    /// One trigger deployment.
    TriggerDeploy,
    /// One deployed trigger deletion.
    TriggerDelete,
    /// The recent events of a deployed trigger.
    TriggerEvents,
    /// The apps enabled in the project environment.
    ProjectInfo,
    /// The Connect usage records.
    Usage,
    /// The project environment webhook.
    Webhook,
}

impl fmt::Display for Target {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Apps => "app list",
            Self::App => "app",
            Self::Categories => "category list",
            Self::Components => "component list",
            Self::Component => "component",
            Self::Actions => "action list",
            Self::ActionRun => "action run",
            Self::TokenCreate => "token creation",
            Self::Accounts => "account list",
            Self::Account => "account",
            Self::Users => "user list",
            Self::ProxyRead => "proxy read",
            Self::ProxyWrite => "proxy write",
            Self::Triggers => "deployed trigger list",
            Self::Trigger => "deployed trigger",
            Self::TriggerDeploy => "trigger deployment",
            Self::TriggerDelete => "trigger deletion",
            Self::TriggerEvents => "trigger event list",
            Self::ProjectInfo => "project info",
            Self::Usage => "usage list",
            Self::Webhook => "project webhook",
        })
    }
}
