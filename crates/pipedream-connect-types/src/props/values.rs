//! Property option and configured-value models.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::{AccountId, PropName};

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
    },
    /// A label-value option nested under `__lv`.
    Nested(Box<PropOption>),
    /// An unlabelled scalar value.
    Scalar(PropOptionValue),
}

impl Serialize for PropOption {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Labelled { label, value } => {
                serde_json::json!({"label": label, "value": value}).serialize(serializer)
            }
            Self::Nested(option) => serde_json::json!({"__lv": option}).serialize(serializer),
            Self::Scalar(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for PropOption {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        if let Value::Object(mut object) = value.clone() {
            if let Some(nested) = object.remove("__lv") {
                let option = match serde_json::from_value(nested) {
                    Ok(option) => option,
                    Err(error) => return Err(serde::de::Error::custom(error)),
                };
                return Ok(Self::Nested(Box::new(option)));
            }
            if let (Some(label), Some(value)) = (object.remove("label"), object.remove("value")) {
                let label = match serde_json::from_value(label) {
                    Ok(label) => label,
                    Err(error) => return Err(serde::de::Error::custom(error)),
                };
                let value = match serde_json::from_value(value) {
                    Ok(value) => value,
                    Err(error) => return Err(serde::de::Error::custom(error)),
                };
                return Ok(Self::Labelled { label, value });
            }
        }
        let value = match serde_json::from_value(value) {
            Ok(value) => value,
            Err(error) => return Err(serde::de::Error::custom(error)),
        };
        Ok(Self::Scalar(value))
    }
}

/// A timer configuration returned in a prop definition.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum TimerValue {
    /// Fixed interval in seconds.
    Interval {
        /// Interval duration.
        #[serde(rename = "intervalSeconds")]
        interval_seconds: i64,
    },
    /// Cron expression.
    Cron {
        /// Cron expression.
        cron: String,
    },
    /// Unknown timer shape.
    Other(Value),
}

impl<'de> Deserialize<'de> for TimerValue {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        if let Some(interval) = value.get("intervalSeconds").and_then(Value::as_i64) {
            return Ok(Self::Interval {
                interval_seconds: interval,
            });
        }
        if let Some(cron) = value.get("cron").and_then(Value::as_str) {
            return Ok(Self::Cron {
                cron: cron.to_owned(),
            });
        }
        Ok(Self::Other(value))
    }
}

/// A configured app, SQL, or arbitrary JSON property value.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum ConfiguredPropValue {
    /// A managed account reference.
    App {
        /// Account identity sent as `authProvisionId`.
        #[serde(rename = "authProvisionId")]
        auth_provision_id: AccountId,
    },
    /// A boolean value.
    Boolean(bool),
    /// A numeric value.
    Integer(f64),
    /// A SQL query value.
    Sql {
        /// Original SQL text.
        value: String,
        /// Prepared SQL query.
        query: String,
        /// Prepared statement parameters.
        params: Vec<String>,
        /// Whether prepared statements are enabled.
        #[serde(rename = "usePreparedStatements")]
        use_prepared_statements: bool,
    },
    /// A string value.
    String(String),
    /// A string array.
    StringArray(Vec<String>),
    /// A JSON object value.
    Object(BTreeMap<String, Value>),
    /// Any JSON shape not covered by a known variant.
    Raw(Value),
}

impl<'de> Deserialize<'de> for ConfiguredPropValue {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        if let Some(object) = value.as_object() {
            if object.contains_key("authProvisionId") {
                let auth_provision_id = match object.get("authProvisionId").cloned() {
                    Some(value) => match serde_json::from_value(value) {
                        Ok(value) => value,
                        Err(error) => return Err(serde::de::Error::custom(error)),
                    },
                    None => return Ok(Self::Raw(value)),
                };
                return Ok(Self::App { auth_provision_id });
            }
            if object.keys().any(|key| key == "usePreparedStatements") {
                let value: SqlValue = match serde_json::from_value(value.clone()) {
                    Ok(value) => value,
                    Err(error) => return Err(serde::de::Error::custom(error)),
                };
                return Ok(Self::Sql {
                    value: value.0,
                    query: value.1,
                    params: value.2,
                    use_prepared_statements: value.3,
                });
            }
            return Ok(Self::Object(object.clone().into_iter().collect()));
        }
        Ok(match value {
            Value::Bool(value) => Self::Boolean(value),
            Value::Number(value) => Self::Integer(value.as_f64().unwrap_or_default()),
            Value::String(value) => Self::String(value),
            Value::Array(values) if values.iter().all(Value::is_string) => Self::StringArray(
                values
                    .into_iter()
                    .filter_map(|value| value.as_str().map(str::to_owned))
                    .collect(),
            ),
            value => Self::Raw(value),
        })
    }
}

/// JSON object of configured properties, keyed by validated property names.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ConfiguredProps(pub BTreeMap<PropName, ConfiguredPropValue>);

impl ConfiguredProps {
    /// Creates an empty configured-property map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts one configured property and returns the previous value.
    pub fn insert(
        &mut self,
        name: PropName,
        value: ConfiguredPropValue,
    ) -> Option<ConfiguredPropValue> {
        self.0.insert(name, value)
    }

    /// Reads one configured property.
    pub fn get(&self, name: &PropName) -> Option<&ConfiguredPropValue> {
        self.0.get(name)
    }

    /// Iterates over configured properties.
    pub fn iter(&self) -> impl Iterator<Item = (&PropName, &ConfiguredPropValue)> {
        self.0.iter()
    }
}

impl<'de> Deserialize<'de> for ConfiguredProps {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        BTreeMap::<PropName, ConfiguredPropValue>::deserialize(deserializer).map(Self)
    }
}

impl From<BTreeMap<PropName, ConfiguredPropValue>> for ConfiguredProps {
    fn from(value: BTreeMap<PropName, ConfiguredPropValue>) -> Self {
        Self(value)
    }
}

/// A typed SQL value used during deserialization without exposing an internal helper.
#[derive(Deserialize)]
struct SqlValue(
    String,
    String,
    Vec<String>,
    #[serde(rename = "usePreparedStatements")] bool,
);
