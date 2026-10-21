//! Exact JSON text for app-defined events and large session identifiers.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::value::RawValue;

use crate::error::PayloadError;

/// Valid JSON whose exact number spelling and object text remain available.
#[derive(Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct RawJson(Box<RawValue>);

impl RawJson {
    /// Parses and stores exact JSON text without number conversion.
    pub fn new(text: impl Into<String>) -> Result<Self, PayloadError> {
        match RawValue::from_string(text.into()) {
            Ok(value) => Ok(Self(value)),
            Err(source) => Err(PayloadError::Json { source }),
        }
    }

    /// Returns the original JSON text.
    pub fn as_str(&self) -> &str {
        self.0.get()
    }

    /// Decodes a caller-selected type from the original JSON text.
    pub fn decode<T: DeserializeOwned>(&self) -> Result<T, PayloadError> {
        match serde_json::from_str(self.as_str()) {
            Ok(value) => Ok(value),
            Err(source) => Err(PayloadError::Json { source }),
        }
    }
}

impl fmt::Debug for RawJson {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl PartialEq for RawJson {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

/// Unknown webhook object fields stored as exact JSON text.
pub type RawFields = BTreeMap<String, RawJson>;

pub(crate) fn required<T, E>(object: &mut RawFields, key: &'static str) -> Result<T, E>
where
    T: DeserializeOwned,
    E: serde::de::Error,
{
    match object.remove(key) {
        Some(raw) => match raw.decode() {
            Ok(value) => Ok(value),
            Err(error) => Err(E::custom(error)),
        },
        None => Err(E::missing_field(key)),
    }
}

pub(crate) fn optional<T, E>(object: &mut RawFields, key: &'static str) -> Result<Option<T>, E>
where
    T: DeserializeOwned,
    E: serde::de::Error,
{
    match object.remove(key) {
        Some(raw) => match raw.decode() {
            Ok(value) => Ok(value),
            Err(error) => Err(E::custom(error)),
        },
        None => Ok(None),
    }
}

#[cfg(test)]
#[path = "_tests_/raw_json_tests.rs"]
mod raw_json_tests;
