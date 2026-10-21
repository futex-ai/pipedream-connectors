//! Lossless trigger delivery and context models.

use chrono::{DateTime, Utc};
use pipedream_connect_types::UnknownFields;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::error::{PayloadError, check_body_limit};

/// Trigger delivery context supplied by Pipedream.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct TriggerContext {
    /// Provider event identifier, when present.
    #[serde(default)]
    pub id: Option<String>,
    /// Provider event timestamp from the `ts` field.
    #[serde(default, rename = "ts", alias = "timestamp")]
    pub timestamp: Option<DateTime<Utc>>,
    /// Unknown context fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// A trigger delivery with a stable event identity.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TriggerDelivery {
    /// App-defined event JSON, preserved without schema conversion.
    pub event: Value,
    /// Provider context and unknown context fields.
    #[serde(default)]
    pub context: TriggerContext,
    /// Context ID or the SHA-256 digest of the raw body when no ID exists.
    #[serde(skip)]
    pub event_id: String,
    /// Unknown top-level fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

impl TriggerDelivery {
    /// Decodes a trigger delivery without applying a body limit.
    pub fn decode(raw_body: &[u8]) -> Result<Self, PayloadError> {
        Self::decode_with_limit(raw_body, usize::MAX)
    }

    /// Checks the body limit before decoding a trigger delivery.
    pub fn decode_with_limit(raw_body: &[u8], body_limit: usize) -> Result<Self, PayloadError> {
        check_body_limit(raw_body, body_limit)?;
        let mut delivery = match serde_json::from_slice::<Self>(raw_body) {
            Ok(delivery) => delivery,
            Err(_) => return Err(PayloadError::InvalidJson),
        };
        delivery.event_id = match delivery.context.id.as_deref() {
            Some(id) if valid_context_id(id) => id.to_owned(),
            Some(_) => return Err(PayloadError::InvalidContextId),
            None => hex::encode(Sha256::digest(raw_body)),
        };
        Ok(delivery)
    }

    /// Returns the stable event identity selected during decoding.
    pub fn identity(&self) -> &str {
        &self.event_id
    }
}

fn valid_context_id(value: &str) -> bool {
    !value.is_empty() && value.len() <= 250 && !value.chars().any(char::is_control)
}

#[cfg(test)]
#[path = "_tests_/trigger_tests.rs"]
mod trigger_tests;
