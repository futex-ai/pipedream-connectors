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
        if !self.ts.is_finite() || self.ts.fract() != 0.0 {
            return Err(Error::InvalidTimestamp);
        }
        crate::Timestamp::from_millis(self.ts as i64).map(|timestamp| timestamp.0)
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
