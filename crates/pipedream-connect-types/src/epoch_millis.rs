//! Serde adapter for timestamps encoded as integer epoch milliseconds.

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serializer};

use crate::Error;

pub(crate) fn serialize<S>(
    date: &DateTime<Utc>,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_i64(date.timestamp_millis())
}

pub(crate) fn deserialize<'de, D>(deserializer: D) -> std::result::Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let millis = i64::deserialize(deserializer)?;
    Utc.timestamp_millis_opt(millis)
        .single()
        .ok_or_else(|| serde::de::Error::custom(Error::InvalidTimestamp))
}
