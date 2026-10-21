//! Shared JSON and timestamp helpers used by response models.

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::error::{Error, Result};

/// Unknown response fields preserved during decoding and re-encoding.
pub type UnknownFields = BTreeMap<String, Value>;

/// An ISO 8601 or epoch-millisecond timestamp decoded to UTC.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Timestamp(pub DateTime<Utc>);

impl Timestamp {
    /// Returns the UTC instant.
    pub fn as_datetime(&self) -> DateTime<Utc> {
        self.0
    }

    /// Decodes an integer epoch-millisecond value.
    pub fn from_millis(value: i64) -> Result<Self> {
        Utc.timestamp_millis_opt(value)
            .single()
            .map(Self)
            .ok_or(Error::InvalidTimestamp)
    }
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        if let Some(number) = value.as_i64() {
            return match Self::from_millis(number) {
                Ok(value) => Ok(value),
                Err(error) => Err(serde::de::Error::custom(error)),
            };
        }
        let text = value
            .as_str()
            .ok_or_else(|| serde::de::Error::custom("timestamp must be a string or integer"))?;
        match DateTime::parse_from_rfc3339(text) {
            Ok(date) => Ok(Self(date.with_timezone(&Utc))),
            Err(error) => Err(serde::de::Error::custom(error)),
        }
    }
}

/// Serde adapter for fields encoded as epoch milliseconds.
pub mod epoch_millis {
    use chrono::{DateTime, TimeZone, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    use crate::error::Error;

    /// Serializes a UTC date as epoch milliseconds.
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(date.timestamp_millis())
    }

    /// Deserializes an epoch-millisecond integer as a UTC date.
    pub fn deserialize<'de, D>(deserializer: D) -> std::result::Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = i64::deserialize(deserializer)?;
        Utc.timestamp_millis_opt(millis)
            .single()
            .ok_or_else(|| serde::de::Error::custom(Error::InvalidTimestamp))
    }
}

/// Serializes an optional epoch-millisecond UTC date.
pub mod option_epoch_millis {
    use chrono::TimeZone;
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serializer};

    use super::epoch_millis;

    /// Serializes an optional UTC date as an optional integer.
    pub fn serialize<S>(
        date: &Option<DateTime<Utc>>,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(date) => epoch_millis::serialize(date, serializer),
            None => serializer.serialize_none(),
        }
    }

    /// Deserializes an optional epoch-millisecond integer.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> std::result::Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<i64>::deserialize(deserializer)?
            .map(|millis| {
                chrono::Utc
                    .timestamp_millis_opt(millis)
                    .single()
                    .ok_or_else(|| serde::de::Error::custom("invalid timestamp"))
            })
            .transpose()
    }
}
