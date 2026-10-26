//! Trigger deployment models and requests.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Result, Timestamp, UnknownFields};

/// One deployed-trigger event.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct EmittedEvent {
    /// Event payload.
    pub e: Value,
    /// Event kind.
    pub k: String,
    /// Event timestamp.
    pub ts: Timestamp,
    /// Event identifier.
    pub id: String,
    /// Optional event summary.
    #[serde(default)]
    pub sum: Option<String>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

impl EmittedEvent {
    /// Returns the event timestamp as a UTC instant.
    pub fn timestamp(&self) -> Result<DateTime<Utc>> {
        Ok(self.ts.as_datetime())
    }
}
