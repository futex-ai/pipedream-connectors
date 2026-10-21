//! Execution models and requests.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A File Stash selector accepted by action execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StashReference {
    /// An existing File Stash identifier.
    Id(String),
    /// Create a new File Stash.
    New,
    /// Enable or disable creation of a new File Stash.
    Boolean(bool),
}

impl Serialize for StashReference {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Id(value) => serializer.serialize_str(value),
            Self::New => serializer.serialize_str("NEW"),
            Self::Boolean(value) => serializer.serialize_bool(*value),
        }
    }
}

impl<'de> Deserialize<'de> for StashReference {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::String(value) if value == "NEW" => Ok(Self::New),
            Value::String(value) => Ok(Self::Id(value)),
            Value::Bool(value) => Ok(Self::Boolean(value)),
            _ => Err(serde::de::Error::custom(
                "stash reference must be a string or boolean",
            )),
        }
    }
}
