//! Execution observations and provider error details.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Error, Result, UnknownFields};

/// One execution observation or log record.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Observation {
    /// Observation error details.
    #[serde(default)]
    pub err: Option<ObservationError>,
    /// Observation source.
    pub k: String,
    /// Log message.
    #[serde(default)]
    pub msg: Option<String>,
    /// Raw epoch-millisecond timestamp.
    pub ts: f64,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

impl Observation {
    /// Converts the raw timestamp to UTC.
    pub fn timestamp(&self) -> Result<DateTime<Utc>> {
        if !self.ts.is_finite() {
            return Err(Error::InvalidTimestamp);
        }
        let seconds = (self.ts / 1000.0).floor();
        if seconds < i64::MIN as f64 || seconds >= i64::MAX as f64 {
            return Err(Error::InvalidTimestamp);
        }
        let nanos = ((self.ts - seconds * 1000.0) * 1_000_000.0).round() as u32;
        let (seconds, nanos) = if nanos == 1_000_000_000 {
            (
                (seconds as i64)
                    .checked_add(1)
                    .ok_or(Error::InvalidTimestamp)?,
                0,
            )
        } else {
            (seconds as i64, nanos)
        };
        DateTime::from_timestamp(seconds, nanos).ok_or(Error::InvalidTimestamp)
    }
}

/// Error details attached to an observation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ObservationError {
    /// Error name.
    #[serde(default)]
    pub name: Option<String>,
    /// Error message.
    #[serde(default)]
    pub message: Option<String>,
    /// Provider stack trace.
    #[serde(default)]
    pub stack: Option<String>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

#[cfg(test)]
#[path = "_tests_/observations_tests.rs"]
mod observations_tests;
