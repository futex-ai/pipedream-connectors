//! Validation and decoding errors for the pure Connect model crate.

use std::fmt;

use internal_error::InternalError;
use thiserror::Error;

/// A bounded request field that failed validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationField {
    /// A cursor page limit.
    Limit,
    /// A Connect token lifetime.
    ExpiresIn,
    /// A trigger event history limit.
    TriggerEventLimit,
}

impl fmt::Display for ValidationField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Limit => "limit",
            Self::ExpiresIn => "expires_in",
            Self::TriggerEventLimit => "n",
        })
    }
}

/// A URI field that must contain an absolute HTTPS URL.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UrlField {
    /// The success redirect URI.
    SuccessRedirectUri,
    /// The error redirect URI.
    ErrorRedirectUri,
    /// The connection webhook URI.
    WebhookUri,
}

impl fmt::Display for UrlField {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::SuccessRedirectUri => "success_redirect_uri",
            Self::ErrorRedirectUri => "error_redirect_uri",
            Self::WebhookUri => "webhook_uri",
        })
    }
}

/// The kind of identifier that failed validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentifierKind {
    /// A project identifier.
    Project,
    /// An external user identifier.
    ExternalUser,
    /// An account identifier.
    Account,
    /// A deployed trigger identifier.
    DeployedTrigger,
    /// An OAuth application identifier.
    OauthApp,
    /// An app category identifier.
    AppCategory,
    /// An app override identifier.
    AppOverride,
    /// An app slug.
    AppSlug,
    /// A component key.
    ComponentKey,
    /// A configurable property name.
    PropName,
    /// A webhook identifier.
    Webhook,
    /// A Connect token.
    ConnectToken,
    /// An opaque cursor.
    Cursor,
    /// A workflow endpoint value.
    WorkflowEndpoint,
}

impl fmt::Display for IdentifierKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Project => "project",
            Self::ExternalUser => "external user",
            Self::Account => "account",
            Self::DeployedTrigger => "deployed trigger",
            Self::OauthApp => "OAuth app",
            Self::AppCategory => "app category",
            Self::AppOverride => "app override",
            Self::AppSlug => "app slug",
            Self::ComponentKey => "component key",
            Self::PropName => "property name",
            Self::Webhook => "webhook",
            Self::ConnectToken => "Connect token",
            Self::Cursor => "cursor",
            Self::WorkflowEndpoint => "workflow endpoint",
        };
        formatter.write_str(name)
    }
}

/// Errors returned while constructing or decoding typed models.
#[derive(Debug, Error, internal_error::ErrorContract)]
pub enum Error {
    /// An identifier does not satisfy its documented grammar.
    #[error("[pipedream-connect-types/ids] invalid {kind} identifier")]
    InvalidIdentifier {
        /// Identifier category.
        kind: IdentifierKind,
    },
    /// A bounded numeric value is outside its documented range.
    #[error("[pipedream-connect-types/validation] {field} must be between {minimum} and {maximum}")]
    OutOfRange {
        /// Field name.
        field: ValidationField,
        /// Inclusive lower bound.
        minimum: u64,
        /// Inclusive upper bound.
        maximum: u64,
    },
    /// A request contains both cursor directions.
    #[error("[pipedream-connect-types/pagination] after and before cannot both be set")]
    ConflictingCursors,
    /// A page response repeats the cursor used to request it.
    #[error("[pipedream-connect-types/pagination] provider returned a stuck cursor")]
    StuckCursor,
    /// A timestamp cannot be represented as a UTC instant.
    #[error("[pipedream-connect-types/time] invalid timestamp")]
    InvalidTimestamp,
    /// A URL must be an absolute HTTPS URL.
    #[error("[pipedream-connect-types/validation] {field} must be an HTTPS URL")]
    InvalidHttpsUrl {
        /// Field containing the URL.
        field: UrlField,
    },
    /// A trigger callback must be HTTPS or loopback HTTP.
    #[error(
        "[pipedream-connect-types/validation] trigger webhook URL must be HTTPS or loopback HTTP"
    )]
    InvalidTriggerWebhookUrl,
    /// An internal serialization or conversion failure.
    #[error("[pipedream-connect-types/internal] internal model error")]
    Internal(#[from] InternalError),
}

/// Result type used by constructors and model helpers.
pub type Result<T> = std::result::Result<T, Error>;
