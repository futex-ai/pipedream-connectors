//! Execution observations and provider error details.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::{Result, Timestamp, UnknownFields};

/// One execution observation or log record.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Observation {
    /// Observation error details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub err: Option<ObservationError>,
    /// Observation source.
    pub k: String,
    /// Log message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
    /// Observation timestamp.
    #[serde(deserialize_with = "deserialize_timestamp_number")]
    pub ts: Timestamp,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

fn deserialize_timestamp_number<'de, D>(deserializer: D) -> std::result::Result<Timestamp, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    if let Some(number) = value.as_number() {
        return match Timestamp::from_number(number.clone()) {
            Ok(timestamp) => Ok(timestamp),
            Err(error) => Err(serde::de::Error::custom(error)),
        };
    }
    match Timestamp::deserialize(value) {
        Ok(timestamp) => Ok(timestamp),
        Err(error) => Err(serde::de::Error::custom(error)),
    }
}

impl Observation {
    /// Converts the raw timestamp to UTC.
    pub fn timestamp(&self) -> Result<DateTime<Utc>> {
        Ok(self.ts.as_datetime())
    }
}

/// Error details attached to an observation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ObservationError {
    /// Error name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Error message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Provider stack trace.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

#[cfg(test)]
#[path = "_tests_/observations_tests.rs"]
mod observations_tests;
