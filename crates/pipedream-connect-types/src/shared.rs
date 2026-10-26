//! Shared JSON and timestamp helpers used by response models.

use std::{collections::BTreeMap, ops::Deref};

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Number, Value};

use crate::error::{Error, Result};

macro_rules! string_enum {
    ($(#[$meta:meta])* pub enum $name:ident {
        $($(#[$variant_meta:meta])* $variant:ident => $wire:literal),+ $(,)?
    }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub enum $name {
            $($(#[$variant_meta])* $variant,)+
            /// A value added by the provider after this SDK version.
            Other(String),
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let value = match self {
                    $(Self::$variant => $wire,)+
                    Self::Other(value) => value.as_str(),
                };
                serializer.serialize_str(value)
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = <String as serde::Deserialize>::deserialize(deserializer)?;
                Ok(match value.as_str() {
                    $($wire => Self::$variant,)+
                    _ => Self::Other(value),
                })
            }
        }
    };
}

pub(crate) use string_enum;

/// Unknown response fields preserved during decoding and re-encoding.
pub type UnknownFields = BTreeMap<String, Value>;

/// An RFC 3339 or epoch-millisecond timestamp decoded to UTC.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Timestamp {
    date: DateTime<Utc>,
    encoding: TimestampEncoding,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum TimestampEncoding {
    Rfc3339,
    EpochMillis(Number),
}

impl Timestamp {
    /// Returns the UTC instant.
    pub fn as_datetime(&self) -> DateTime<Utc> {
        self.date
    }

    /// Returns the instant as epoch milliseconds.
    pub fn as_millis(&self) -> i64 {
        self.date.timestamp_millis()
    }

    /// Creates an RFC 3339 timestamp from a UTC instant.
    pub fn from_datetime(date: DateTime<Utc>) -> Self {
        Self {
            date,
            encoding: TimestampEncoding::Rfc3339,
        }
    }

    /// Decodes an integer epoch-millisecond value.
    pub fn from_millis(value: i64) -> Result<Self> {
        Utc.timestamp_millis_opt(value)
            .single()
            .map(|date| Self {
                date,
                encoding: TimestampEncoding::EpochMillis(Number::from(value)),
            })
            .ok_or(Error::InvalidTimestamp)
    }

    /// Decodes an epoch-millisecond JSON number.
    pub fn from_number(value: Number) -> Result<Self> {
        if let Some(millis) = value.as_i64() {
            return match Self::from_millis(millis) {
                Ok(mut timestamp) => {
                    timestamp.encoding = TimestampEncoding::EpochMillis(value);
                    Ok(timestamp)
                }
                Err(error) => Err(error),
            };
        }
        let Some(millis) = value.as_f64() else {
            return Err(Error::InvalidTimestamp);
        };
        if !millis.is_finite() {
            return Err(Error::InvalidTimestamp);
        }
        let seconds = (millis / 1000.0).floor();
        if seconds < i64::MIN as f64 || seconds >= i64::MAX as f64 {
            return Err(Error::InvalidTimestamp);
        }
        let nanos = ((millis - seconds * 1000.0) * 1_000_000.0).round() as u32;
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
        let date = DateTime::from_timestamp(seconds, nanos).ok_or(Error::InvalidTimestamp)?;
        Ok(Self {
            date,
            encoding: TimestampEncoding::EpochMillis(value),
        })
    }
}

impl Deref for Timestamp {
    type Target = DateTime<Utc>;

    fn deref(&self) -> &Self::Target {
        &self.date
    }
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.encoding {
            TimestampEncoding::Rfc3339 => self.date.serialize(serializer),
            TimestampEncoding::EpochMillis(value) => value.serialize(serializer),
        }
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
            Ok(date) => Ok(Self::from_datetime(date.with_timezone(&Utc))),
            Err(error) => Err(serde::de::Error::custom(error)),
        }
    }
}

#[cfg(test)]
#[path = "_tests_/shared_tests.rs"]
mod shared_tests;

#[cfg(test)]
#[path = "_tests_/timestamp_tests.rs"]
mod timestamp_tests;
