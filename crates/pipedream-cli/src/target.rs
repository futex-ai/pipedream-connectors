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
        })
    }
}
