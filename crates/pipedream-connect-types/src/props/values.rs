//! Configured property values and maps.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Number, Value};

use crate::{AccountId, PropName, UnknownFields};

use super::serde_helpers::{encode, insert_extra};

/// A configured app, SQL, or arbitrary JSON property value.
#[derive(Clone, Debug, PartialEq)]
pub enum ConfiguredPropValue {
    /// A managed account reference.
    App {
        /// Account identity sent as `authProvisionId`.
        auth_provision_id: AccountId,
        /// Unknown fields in the app object.
        extra: UnknownFields,
    },
    /// A boolean value.
    Boolean(bool),
    /// A lossless JSON number.
    Number(Number),
    /// A SQL query value.
    Sql {
        /// Original SQL text.
        value: String,
        /// Prepared SQL query.
        query: String,
        /// Prepared statement parameters.
        params: Vec<String>,
        /// Whether prepared statements are enabled.
        use_prepared_statements: bool,
        /// Unknown fields in the SQL object.
        extra: UnknownFields,
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

impl Serialize for ConfiguredPropValue {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = match self {
            Self::App {
                auth_provision_id,
                extra,
            } => {
                let mut object = Map::new();
                object.insert("authProvisionId".into(), encode(auth_provision_id)?);
                insert_extra(&mut object, extra);
                Value::Object(object)
            }
            Self::Boolean(value) => Value::Bool(*value),
            Self::Number(value) => Value::Number(value.clone()),
            Self::Sql {
                value,
                query,
                params,
                use_prepared_statements,
                extra,
            } => {
                let mut object = Map::new();
                object.insert("value".into(), Value::String(value.clone()));
                object.insert("query".into(), Value::String(query.clone()));
                object.insert("params".into(), encode(params)?);
                object.insert(
                    "usePreparedStatements".into(),
                    Value::Bool(*use_prepared_statements),
                );
                insert_extra(&mut object, extra);
                Value::Object(object)
            }
            Self::String(value) => Value::String(value.clone()),
            Self::StringArray(value) => encode(value)?,
            Self::Object(value) => Value::Object(value.clone().into_iter().collect()),
            Self::Raw(value) => value.clone(),
        };
        value.serialize(serializer)
    }
}

#[derive(Deserialize)]
struct SqlValue {
    value: String,
    query: String,
    params: Vec<String>,
    #[serde(rename = "usePreparedStatements")]
    use_prepared_statements: bool,
    #[serde(flatten)]
    extra: UnknownFields,
}

impl<'de> Deserialize<'de> for ConfiguredPropValue {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        if let Value::Object(mut object) = value {
            if let Some(account) = object.remove("authProvisionId") {
                let Ok(auth_provision_id) = serde_json::from_value(account.clone()) else {
                    let mut raw = Map::new();
                    raw.insert("authProvisionId".into(), account);
                    raw.extend(object);
                    return Ok(Self::Raw(Value::Object(raw)));
                };
                return Ok(Self::App {
                    auth_provision_id,
                    extra: object.into_iter().collect(),
                });
            }
            if object.contains_key("usePreparedStatements") {
                let raw = Value::Object(object);
                let Ok(sql) = serde_json::from_value::<SqlValue>(raw.clone()) else {
                    return Ok(Self::Raw(raw));
                };
                return Ok(Self::Sql {
                    value: sql.value,
                    query: sql.query,
                    params: sql.params,
                    use_prepared_statements: sql.use_prepared_statements,
                    extra: sql.extra,
                });
            }
            return Ok(Self::Object(object.into_iter().collect()));
        }
        Ok(match value {
            Value::Bool(value) => Self::Boolean(value),
            Value::Number(value) => Self::Number(value),
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
pub struct ConfiguredProps(
    /// Configured values keyed by their exact property names.
    pub BTreeMap<PropName, ConfiguredPropValue>,
);

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
