//! Serde adapter for optional epoch-millisecond timestamps.

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Deserializer, Serializer};

use crate::{Error, epoch_millis};

pub(crate) fn serialize<S>(
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

pub(crate) fn deserialize<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<i64>::deserialize(deserializer)?
        .map(|millis| {
            Utc.timestamp_millis_opt(millis)
                .single()
                .ok_or_else(|| serde::de::Error::custom(Error::InvalidTimestamp))
        })
        .transpose()
}
