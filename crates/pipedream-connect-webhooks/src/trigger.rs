//! Lossless trigger delivery and context models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{PayloadError, check_body_limit};
use crate::raw_json::{RawFields, RawJson, optional};

/// Trigger delivery context supplied by Pipedream.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct TriggerContext {
    /// Provider event identifier, when present.
    #[serde(default)]
    pub id: Option<String>,
    /// Provider event timestamp from the `ts` field.
    #[serde(default, rename = "ts", alias = "timestamp")]
    pub timestamp: Option<DateTime<Utc>>,
    /// Unknown context fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: RawFields,
}

/// A trigger delivery with a stable event identity.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TriggerDelivery {
    /// App-defined event JSON, preserved without schema conversion.
    pub event: RawJson,
    /// Provider context and unknown context fields.
    #[serde(default)]
    pub context: TriggerContext,
    /// Context ID or the SHA-256 digest of the raw body when no ID exists.
    #[serde(skip)]
    event_id: String,
    /// Unknown top-level fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: RawFields,
}

impl<'de> Deserialize<'de> for TriggerContext {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut fields = RawFields::deserialize(deserializer)?;
        let id = optional(&mut fields, "id")?;
        let timestamp = if fields.contains_key("ts") {
            optional(&mut fields, "ts")?
        } else {
            optional(&mut fields, "timestamp")?
        };
        Ok(Self {
            id,
            timestamp,
            extra: fields,
        })
    }
}

impl<'de> Deserialize<'de> for TriggerDelivery {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawJson::deserialize(deserializer)?;
        let mut fields: RawFields = match raw.decode() {
            Ok(fields) => fields,
            Err(error) => return Err(serde::de::Error::custom(error)),
        };
        let event = fields
            .remove("event")
            .ok_or_else(|| serde::de::Error::missing_field("event"))?;
        let context: TriggerContext = optional(&mut fields, "context")?.unwrap_or_default();
        let event_id = context
            .id
            .clone()
            .unwrap_or_else(|| hex::encode(Sha256::digest(raw.as_str().as_bytes())));
        Ok(Self {
            event,
            context,
            event_id,
            extra: fields,
        })
    }
}

impl TriggerDelivery {
    /// Decodes a trigger delivery without applying a body limit.
    pub fn decode(raw_body: &[u8]) -> Result<Self, PayloadError> {
        Self::decode_with_limit(raw_body, usize::MAX)
    }

    /// Checks the body limit before decoding a trigger delivery.
    pub fn decode_with_limit(raw_body: &[u8], body_limit: usize) -> Result<Self, PayloadError> {
        check_body_limit(raw_body, body_limit)?;
        let delivery = match serde_json::from_slice::<Self>(raw_body) {
            Ok(delivery) => delivery,
            Err(_) => return Err(PayloadError::InvalidJson),
        };
        if delivery
            .context
            .id
            .as_deref()
            .is_some_and(|id| !valid_context_id(id))
        {
            return Err(PayloadError::InvalidContextId);
        }
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
