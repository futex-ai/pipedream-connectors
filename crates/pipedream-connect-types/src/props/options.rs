//! Property option values and lossless option wrappers.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

use crate::UnknownFields;

use super::serde_helpers::{decode, encode, insert_extra};

/// A scalar option value from the provider.
#[derive(Clone, Debug, PartialEq)]
pub enum PropOptionValue {
    /// Text option.
    String(String),
    /// Integer option.
    Integer(i64),
    /// Boolean option.
    Boolean(bool),
    /// Explicit JSON null.
    Null,
    /// A scalar shape added by the provider.
    Other(Value),
}

impl Serialize for PropOptionValue {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::String(value) => serializer.serialize_str(value),
            Self::Integer(value) => serializer.serialize_i64(*value),
            Self::Boolean(value) => serializer.serialize_bool(*value),
            Self::Null => serializer.serialize_none(),
            Self::Other(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for PropOptionValue {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Ok(match value {
            Value::String(value) => Self::String(value),
            Value::Number(value) => value
                .as_i64()
                .map(Self::Integer)
                .unwrap_or_else(|| Self::Other(Value::Number(value))),
            Value::Bool(value) => Self::Boolean(value),
            Value::Null => Self::Null,
            value => Self::Other(value),
        })
    }
}

/// A labelled, nested, or scalar property option.
#[derive(Clone, Debug, PartialEq)]
pub enum PropOption {
    /// A human-readable label and scalar or JSON value.
    Labelled {
        /// Display label.
        label: String,
        /// Exact option value.
        value: PropOptionValue,
        /// Unknown fields in the option object.
        extra: UnknownFields,
    },
    /// A label-value option nested under `__lv`.
    Nested {
        /// Nested option.
        option: Box<PropOption>,
        /// Unknown fields in the wrapper object.
        extra: UnknownFields,
    },
    /// An unlabelled scalar value.
    Scalar(PropOptionValue),
}

impl Serialize for PropOption {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match self {
            Self::Labelled {
                label,
                value,
                extra,
            } => {
                let mut object = Map::new();
                object.insert("label".into(), Value::String(label.clone()));
                object.insert("value".into(), encode(value)?);
                insert_extra(&mut object, extra);
                Value::Object(object)
            }
            Self::Nested { option, extra } => {
                let mut object = Map::new();
                object.insert("__lv".into(), encode(option)?);
                insert_extra(&mut object, extra);
                Value::Object(object)
            }
            Self::Scalar(value) => encode(value)?,
        };
        value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PropOption {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let Value::Object(mut object) = value.clone() else {
            return scalar(value);
        };
        if let Some(nested) = object.remove("__lv") {
            let option = decode(nested)?;
            return Ok(Self::Nested {
                option: Box::new(option),
                extra: object.into_iter().collect(),
            });
        }
        let Some(label_value) = object.remove("label") else {
            return scalar(Value::Object(object));
        };
        let Some(option_value) = object.remove("value") else {
            return scalar(Value::Object({
                let mut value = Map::new();
                value.insert("label".into(), label_value);
                value.extend(object);
                value
            }));
        };
        let Some(label) = label_value.as_str() else {
            let mut raw = Map::new();
            raw.insert("label".into(), label_value);
            raw.insert("value".into(), option_value);
            raw.extend(object);
            return scalar(Value::Object(raw));
        };
        let option_value = decode(option_value)?;
        Ok(Self::Labelled {
            label: label.to_owned(),
            value: option_value,
            extra: object.into_iter().collect(),
        })
    }
}

fn scalar<E>(value: Value) -> std::result::Result<PropOption, E>
where
    E: serde::de::Error,
{
    decode(value).map(PropOption::Scalar)
}
