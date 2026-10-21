//! The command arguments that the CLI validates before any request.

use std::fmt;

/// A command argument that the CLI validates before any request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Argument {
    /// The page size or event count flag.
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
    /// The registry component to read or deploy.
    Component,
    /// The action to run.
    Action,
    /// The account to read.
    Account,
    /// The account flag of a proxy request.
    AccountFlag,
    /// The deployed trigger to read or change.
    Trigger,
    /// The external user flag.
    ExternalUserId,
    /// The OAuth app filter.
    OauthAppId,
    /// The configured props input.
    Props,
    /// The proxy body input.
    Body,
    /// The Connect token lifetime.
    ExpiresIn,
    /// The Connect success redirect.
    SuccessRedirectUri,
    /// The Connect error redirect.
    ErrorRedirectUri,
    /// The Connect webhook URI.
    WebhookUri,
    /// The deployed trigger webhook URL.
    WebhookUrl,
    /// The deployed trigger workflow.
    WorkflowId,
    /// The start of a usage window.
    Start,
    /// The end of a usage window.
    End,
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
            Self::Account => "<ACCOUNT>",
            Self::AccountFlag => "--account",
            Self::Trigger => "<TRIGGER>",
            Self::ExternalUserId => "--external-user-id",
            Self::OauthAppId => "--oauth-app-id",
            Self::Props => "--props",
            Self::Body => "--body",
            Self::ExpiresIn => "--expires-in",
            Self::SuccessRedirectUri => "--success-redirect-uri",
            Self::ErrorRedirectUri => "--error-redirect-uri",
            Self::WebhookUri => "--webhook-uri",
            Self::WebhookUrl => "--webhook-url",
            Self::WorkflowId => "--workflow-id",
            Self::Start => "--start",
            Self::End => "--end",
        }
    }
}

impl fmt::Display for Argument {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}
