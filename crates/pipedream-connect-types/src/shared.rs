//! Shared JSON and timestamp helpers used by response models.

use std::collections::BTreeMap;

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

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

/// An ISO 8601 or epoch-millisecond timestamp decoded to UTC.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Timestamp(
    /// The decoded UTC instant.
    pub DateTime<Utc>,
);

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

#[cfg(test)]
#[path = "_tests_/shared_tests.rs"]
mod shared_tests;
