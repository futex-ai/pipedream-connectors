//! Lossless timer configuration values.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

use crate::UnknownFields;

use super::serde_helpers::insert_extra;

/// A timer configuration returned in a prop definition.
#[derive(Clone, Debug, PartialEq)]
pub enum TimerValue {
    /// Fixed interval in seconds.
    Interval {
        /// Interval duration.
        interval_seconds: i64,
        /// Unknown fields in the interval object.
        extra: UnknownFields,
    },
    /// Cron expression.
    Cron {
        /// Cron expression.
        cron: String,
        /// Unknown fields in the cron object.
        extra: UnknownFields,
    },
    /// Unknown timer shape.
    Other(Value),
}

impl Serialize for TimerValue {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match self {
            Self::Interval {
                interval_seconds,
                extra,
            } => {
                let mut object = Map::new();
                object.insert("intervalSeconds".into(), Value::from(*interval_seconds));
                insert_extra(&mut object, extra);
                Value::Object(object)
            }
            Self::Cron { cron, extra } => {
                let mut object = Map::new();
                object.insert("cron".into(), Value::String(cron.clone()));
                insert_extra(&mut object, extra);
                Value::Object(object)
            }
            Self::Other(value) => value.clone(),
        };
        value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TimerValue {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let Value::Object(mut object) = value.clone() else {
            return Ok(Self::Other(value));
        };
        if let Some(interval) = object.remove("intervalSeconds") {
            if let Some(interval) = interval.as_i64() {
                return Ok(Self::Interval {
                    interval_seconds: interval,
                    extra: object.into_iter().collect(),
                });
            }
            return Ok(Self::Other({
                let mut raw = Map::new();
                raw.insert("intervalSeconds".into(), interval);
                raw.extend(object);
                Value::Object(raw)
            }));
        }
        if let Some(cron) = object.remove("cron") {
            if let Some(cron) = cron.as_str() {
                return Ok(Self::Cron {
                    cron: cron.to_owned(),
                    extra: object.into_iter().collect(),
                });
            }
            return Ok(Self::Other({
                let mut raw = Map::new();
                raw.insert("cron".into(), cron);
                raw.extend(object);
                Value::Object(raw)
            }));
        }
        Ok(Self::Other(Value::Object(object)))
    }
}
