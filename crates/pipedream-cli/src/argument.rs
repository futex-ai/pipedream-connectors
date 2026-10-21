//! The command arguments that the CLI validates before any request.

use std::fmt;

/// A command argument that the CLI validates before any request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Argument {
    /// The page size flag.
    Limit,
    /// The page budget of `--all`.
    MaxPages,
    /// The forward cursor flag.
    After,
    /// The backward cursor flag.
    Before,
    /// An app category filter.
    Category,
    /// An app filter.
    AppFilter,
    /// The app to read.
    App,
    /// The component to read.
    Component,
    /// The action to run.
    Action,
    /// The external user flag.
    ExternalUserId,
    /// The configured props input.
    Props,
}

impl Argument {
    /// Returns the argument as the help text shows it.
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Limit => "--limit",
            Self::MaxPages => "--max-pages",
            Self::After => "--after",
            Self::Before => "--before",
            Self::Category => "--category",
            Self::AppFilter => "--app",
            Self::App => "<APP>",
            Self::Component => "<COMPONENT>",
            Self::Action => "<ACTION>",
            Self::ExternalUserId => "--external-user-id",
            Self::Props => "--props",
        }
    }
}

impl fmt::Display for Argument {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}
