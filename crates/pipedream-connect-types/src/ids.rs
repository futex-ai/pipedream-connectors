//! Validated identifiers used in Connect paths and request bodies.

#![allow(
    clippy::disallowed_methods,
    reason = "serde's try_from derives generate map_err"
)]

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::{Error, IdentifierKind, Result};

/// Returns whether a value uses the common path-safe identifier grammar.
pub(crate) fn common_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 250
        && !matches!(value, "." | "..")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && !matches!(byte, b'/' | b'\\' | b'?' | b'#'))
}

/// Returns whether a value uses the opaque cursor grammar.
pub(crate) fn cursor_identifier(value: &str) -> bool {
    !value.is_empty() && value.len() <= 2048 && value.bytes().all(|byte| byte.is_ascii_graphic())
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
    "A deployed trigger identifier with a documented `dc_`, `hi_`, or `ti_` prefix.",
    |value: &str| ["dc_", "hi_", "ti_"]
        .iter()
        .any(|prefix| prefixed_alphanumeric(value, prefix))
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
    "A provider app slug using the common path-safe grammar.",
    common_identifier
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
    "An opaque pagination cursor using visible ASCII up to 2048 bytes.",
    cursor_identifier
);

identifier!(
    AppOverrideId,
    AppOverride,
    "An app override identifier using the common path-safe grammar.",
    common_identifier
);
identifier!(
    WorkflowEndpoint,
    WorkflowEndpoint,
    "An opaque workflow endpoint value using the common path-safe grammar.",
    common_identifier
);

#[cfg(test)]
#[path = "_tests_/ids_tests.rs"]
mod ids_tests;
