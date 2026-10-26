//! Execution models and requests.

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::DeserializeOwned};
use serde_json::{Map, Number, Value};

use crate::{UnknownFields, string_enum};

string_enum! {
    /// The stage that caused a provider execution failure.
    pub enum ErrorOrigin {
        /// Component code failed.
        ComponentCode => "component_code",
        /// The upstream API returned an error.
        UpstreamApi => "upstream_api",
        /// Network I/O failed.
        NetworkIo => "network_io",
        /// Response parsing failed.
        ResponseParsing => "response_parsing",
    }
}

/// Details about the upstream calls made by an execution.
#[derive(Clone, Debug, PartialEq)]
pub struct ExecutionAttribution {
    /// Failure stage.
    pub origin: ErrorOrigin,
    /// Number of upstream calls.
    pub network_calls_count: Option<u64>,
    /// Last upstream call metadata.
    pub last_call: Option<NetworkCall>,
    /// Unknown attribution fields.
    pub extra: UnknownFields,
}

impl<'de> Deserialize<'de> for ExecutionAttribution {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut object = object(deserializer, "execution attribution must be an object")?;
        let mut extra = UnknownFields::new();
        let origin = match object.remove("origin") {
            Some(value) => match serde_json::from_value(value.clone()) {
                Ok(origin) => origin,
                Err(error) => return Err(serde::de::Error::custom(error)),
            },
            None => return Err(serde::de::Error::missing_field("origin")),
        };
        let network_calls_count = take_optional(&mut object, &mut extra, "network_calls_count");
        let last_call = take_optional(&mut object, &mut extra, "last_call");
        extra.extend(object);
        Ok(Self {
            origin,
            network_calls_count,
            last_call,
            extra,
        })
    }
}

impl Serialize for ExecutionAttribution {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut object = Map::new();
        if !self.extra.contains_key("origin") {
            insert_value(&mut object, "origin", &self.origin)?;
        }
        insert_optional(
            &mut object,
            &self.extra,
            "network_calls_count",
            &self.network_calls_count,
        )?;
        insert_optional(&mut object, &self.extra, "last_call", &self.last_call)?;
        insert_extra(&mut object, &self.extra);
        Value::Object(object).serialize(serializer)
    }
}

/// Metadata for the last upstream call.
#[derive(Clone, Debug, PartialEq)]
pub struct NetworkCall {
    /// Upstream client name.
    pub client: Option<String>,
    /// Upstream HTTP method.
    pub method: Option<String>,
    /// Upstream host.
    pub host: Option<String>,
    /// Upstream status.
    pub status: Option<u16>,
    /// Upstream duration in milliseconds.
    pub duration_ms: Option<Number>,
    /// Unknown call fields.
    pub extra: UnknownFields,
}

impl<'de> Deserialize<'de> for NetworkCall {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut object = object(deserializer, "network call must be an object")?;
        let mut extra = UnknownFields::new();
        let client = take_optional(&mut object, &mut extra, "client");
        let method = take_optional(&mut object, &mut extra, "method");
        let host = take_optional(&mut object, &mut extra, "host");
        let status = take_optional(&mut object, &mut extra, "status");
        let duration_ms = take_optional(&mut object, &mut extra, "duration_ms");
        extra.extend(object);
        Ok(Self {
            client,
            method,
            host,
            status,
            duration_ms,
            extra,
        })
    }
}

impl Serialize for NetworkCall {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut object = Map::new();
        insert_optional(&mut object, &self.extra, "client", &self.client)?;
        insert_optional(&mut object, &self.extra, "method", &self.method)?;
        insert_optional(&mut object, &self.extra, "host", &self.host)?;
        insert_optional(&mut object, &self.extra, "status", &self.status)?;
        insert_optional(&mut object, &self.extra, "duration_ms", &self.duration_ms)?;
        insert_extra(&mut object, &self.extra);
        Value::Object(object).serialize(serializer)
    }
}

/// A structured provider execution failure.
#[derive(Clone, Debug, PartialEq)]
pub struct ExecutionError {
    /// Failure attribution.
    pub attribution: Option<ExecutionAttribution>,
    /// Provider error name.
    pub name: Option<String>,
    /// Provider error message.
    pub message: Option<String>,
    /// Unknown provider fields.
    pub extra: UnknownFields,
}

impl<'de> Deserialize<'de> for ExecutionError {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut object = object(deserializer, "execution error must be an object")?;
        let mut extra = UnknownFields::new();
        let attribution = take_optional(&mut object, &mut extra, "attribution");
        let name = take_optional(&mut object, &mut extra, "name");
        let message = take_optional(&mut object, &mut extra, "message");
        extra.extend(object);
        Ok(Self {
            attribution,
            name,
            message,
            extra,
        })
    }
}

impl Serialize for ExecutionError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut object = Map::new();
        insert_optional(&mut object, &self.extra, "attribution", &self.attribution)?;
        insert_optional(&mut object, &self.extra, "name", &self.name)?;
        insert_optional(&mut object, &self.extra, "message", &self.message)?;
        insert_extra(&mut object, &self.extra);
        Value::Object(object).serialize(serializer)
    }
}

fn object<'de, D>(
    deserializer: D,
    message: &'static str,
) -> std::result::Result<Map<String, Value>, D::Error>
where
    D: Deserializer<'de>,
{
    match Value::deserialize(deserializer)? {
        Value::Object(object) => Ok(object),
        _ => Err(serde::de::Error::custom(message)),
    }
}

fn take_optional<T: DeserializeOwned>(
    object: &mut Map<String, Value>,
    extra: &mut UnknownFields,
    key: &str,
) -> Option<T> {
    let value = object.remove(key)?;
    if value.is_null() {
        extra.insert(key.to_owned(), value);
        return None;
    }
    match serde_json::from_value(value.clone()) {
        Ok(value) => Some(value),
        Err(_) => {
            extra.insert(key.to_owned(), value);
            None
        }
    }
}

fn insert_optional<T: Serialize, E: serde::ser::Error>(
    object: &mut Map<String, Value>,
    extra: &UnknownFields,
    key: &str,
    value: &Option<T>,
) -> std::result::Result<(), E> {
    if !extra.contains_key(key)
        && let Some(value) = value
    {
        insert_value(object, key, value)?;
    }
    Ok(())
}

fn insert_value<T: Serialize, E: serde::ser::Error>(
    object: &mut Map<String, Value>,
    key: &str,
    value: &T,
) -> std::result::Result<(), E> {
    let value = match serde_json::to_value(value) {
        Ok(value) => value,
        Err(error) => return Err(E::custom(error)),
    };
    object.insert(key.to_owned(), value);
    Ok(())
}

fn insert_extra(object: &mut Map<String, Value>, extra: &UnknownFields) {
    for (key, value) in extra {
        object.entry(key.clone()).or_insert_with(|| value.clone());
    }
}
