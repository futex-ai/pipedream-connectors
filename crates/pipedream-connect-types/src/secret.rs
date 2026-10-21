//! Credential wrappers that require explicit access and redact diagnostics.

use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Error, Result};

/// A string that serializes on the wire but never exposes itself in `Debug`.
#[derive(Clone, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SecretString(String);

impl SecretString {
    /// Wraps a credential-class string.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the wrapped string for an explicitly authorized operation.
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[REDACTED]")
    }
}

/// Arbitrary JSON that serializes on the wire but redacts its diagnostics.
#[derive(Clone, Default, Deserialize, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SecretJson(Value);

impl SecretJson {
    /// Wraps a credential-class JSON value.
    pub fn new(value: Value) -> Self {
        Self(value)
    }

    /// Returns the wrapped JSON value for an explicitly authorized operation.
    pub fn expose_secret(&self) -> &Value {
        &self.0
    }
}

impl fmt::Debug for SecretJson {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[REDACTED]")
    }
}

/// A short-lived Connect token with a strict wire grammar.
#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ConnectToken(String);

impl ConnectToken {
    /// Validates a `ctok_` token with 32 lowercase hexadecimal digits.
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        let valid = value.strip_prefix("ctok_").is_some_and(|suffix| {
            suffix.len() == 32
                && suffix
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        });
        if valid {
            Ok(Self(value))
        } else {
            Err(Error::InvalidIdentifier {
                kind: crate::IdentifierKind::ConnectToken,
            })
        }
    }

    /// Returns the exact token for a connection handoff.
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ConnectToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[REDACTED]")
    }
}

impl<'de> Deserialize<'de> for ConnectToken {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match Self::new(String::deserialize(deserializer)?) {
            Ok(value) => Ok(value),
            Err(error) => Err(serde::de::Error::custom(error)),
        }
    }
}

impl TryFrom<String> for ConnectToken {
    type Error = Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// The Connect project environment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Environment {
    /// Development environment.
    Development,
    /// Production environment.
    Production,
    /// Provider value added after this SDK version.
    Other(String),
}

impl Serialize for Environment {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = match self {
            Self::Development => "development",
            Self::Production => "production",
            Self::Other(value) => value.as_str(),
        };
        serializer.serialize_str(value)
    }
}

impl<'de> Deserialize<'de> for Environment {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "development" => Self::Development,
            "production" => Self::Production,
            _ => Self::Other(value),
        })
    }
}
