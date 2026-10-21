//! Validated identifiers used in Connect paths and request bodies.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::{Error, IdentifierKind, Result};

/// Returns whether a value uses the common path-safe identifier grammar.
pub(crate) fn common_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 250
        && value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && !matches!(byte, b'/' | b'\\' | b'?' | b'#'))
}

/// Returns whether a value has the supplied ASCII-alphanumeric prefix.
pub(crate) fn prefixed_alphanumeric(value: &str, prefix: &str) -> bool {
    common_identifier(value)
        && value.strip_prefix(prefix).is_some_and(|suffix| {
            !suffix.is_empty() && suffix.bytes().all(|byte| byte.is_ascii_alphanumeric())
        })
}

macro_rules! identifier {
    ($name:ident, $kind:ident, $doc:literal, $validator:expr) => {
        #[doc = $doc]
        #[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(String);

        impl $name {
            /// Validates the exact identifier without trimming or normalization.
            pub fn new(value: impl Into<String>) -> Result<Self> {
                let value = value.into();
                if ($validator)(&value) {
                    Ok(Self(value))
                } else {
                    Err(Error::InvalidIdentifier {
                        kind: IdentifierKind::$kind,
                    })
                }
            }

            /// Borrows the exact identifier.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = Error;

            fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

identifier!(
    ProjectId,
    Project,
    "A project identifier with the documented `proj_` prefix.",
    |value: &str| prefixed_alphanumeric(value, "proj_")
);
identifier!(
    ExternalUserId,
    ExternalUser,
    "An external user identifier bounded by the common path-safe grammar.",
    common_identifier
);
identifier!(
    AccountId,
    Account,
    "An account identifier with the documented `apn_` prefix.",
    |value: &str| prefixed_alphanumeric(value, "apn_")
);
identifier!(
    DeployedTriggerId,
    DeployedTrigger,
    "A deployed trigger identifier with the documented `dc_` prefix.",
    |value: &str| prefixed_alphanumeric(value, "dc_")
);
identifier!(
    OauthAppId,
    OauthApp,
    "A custom OAuth application identifier with the documented `oa_` prefix.",
    |value: &str| prefixed_alphanumeric(value, "oa_")
);
identifier!(
    AppCategoryId,
    AppCategory,
    "An app category identifier with the documented `appcat_` prefix.",
    |value: &str| prefixed_alphanumeric(value, "appcat_")
);
identifier!(
    AppSlug,
    AppSlug,
    "A lowercase provider app slug.",
    |value: &str| common_identifier(value)
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-')
        })
);
identifier!(
    ComponentKey,
    ComponentKey,
    "A registry component key using path-safe visible ASCII.",
    common_identifier
);
identifier!(
    PropName,
    PropName,
    "A configurable property name using path-safe visible ASCII.",
    common_identifier
);
identifier!(
    WebhookId,
    Webhook,
    "A webhook identifier using the common path-safe grammar.",
    common_identifier
);
identifier!(
    Cursor,
    Cursor,
    "An opaque pagination cursor using the common path-safe grammar.",
    common_identifier
);

/// An opaque workflow endpoint value accepted by the API.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct WorkflowEndpoint(String);

impl WorkflowEndpoint {
    /// Validates a workflow endpoint value, including documented `p_` IDs.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if common_identifier(&value) {
            Ok(Self(value))
        } else {
            Err(Error::InvalidIdentifier {
                kind: IdentifierKind::WorkflowEndpoint,
            })
        }
    }

    /// Borrows the endpoint value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for WorkflowEndpoint {
    type Error = Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<WorkflowEndpoint> for String {
    fn from(value: WorkflowEndpoint) -> Self {
        value.0
    }
}

impl fmt::Display for WorkflowEndpoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
