//! Trigger emitter variants and lossless discriminator handling.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Number, Value};

use crate::{
    ComponentKey, ConfigurableProp, ConfiguredProps, DeployedTriggerId, SecretString,
    TriggerWebhook, UnknownFields, epoch_millis, string_enum,
};

/// A deployed component emitter.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DeployedComponent {
    /// Deployed trigger identifier.
    pub id: DeployedTriggerId,
    /// Provider owner identifier.
    pub owner_id: String,
    /// Component identifier.
    pub component_id: String,
    /// Component key when supplied.
    #[serde(default)]
    pub component_key: Option<ComponentKey>,
    /// Configurable properties.
    pub configurable_props: Vec<ConfigurableProp>,
    /// Configured property values.
    pub configured_props: ConfiguredProps,
    /// Whether the deployment is active.
    pub active: bool,
    /// Deployment timestamp in epoch milliseconds.
    #[serde(
        deserialize_with = "epoch_millis::deserialize",
        serialize_with = "epoch_millis::serialize"
    )]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp in epoch milliseconds.
    #[serde(
        deserialize_with = "epoch_millis::deserialize",
        serialize_with = "epoch_millis::serialize"
    )]
    pub updated_at: DateTime<Utc>,
    /// Display name.
    pub name: String,
    /// Display slug.
    pub name_slug: String,
    /// Callback observations.
    #[serde(default)]
    pub callback_observations: Option<Value>,
    /// Whether deploy hooks may emit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emit_on_deploy: Option<bool>,
    /// Discriminator.
    #[serde(rename = "type")]
    pub type_name: String,
    /// Signing key returned for an authenticated deployment.
    #[serde(default)]
    pub webhook_signing_key: Option<SecretString>,
    /// Effective webhook destination URLs when returned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webhook_urls: Option<Vec<String>>,
    /// Webhook destinations and redacted signing keys when returned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webhooks: Option<Vec<TriggerWebhook>>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// An HTTP interface emitter.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct HttpInterface {
    /// Interface identifier.
    pub id: String,
    /// Interface key.
    pub key: String,
    /// Endpoint URL.
    pub endpoint_url: String,
    /// Whether custom responses are enabled.
    pub custom_response: bool,
    /// Creation timestamp in epoch milliseconds.
    #[serde(
        deserialize_with = "epoch_millis::deserialize",
        serialize_with = "epoch_millis::serialize"
    )]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp in epoch milliseconds.
    #[serde(
        deserialize_with = "epoch_millis::deserialize",
        serialize_with = "epoch_millis::serialize"
    )]
    pub updated_at: DateTime<Utc>,
    /// Discriminator.
    #[serde(rename = "type")]
    pub type_name: String,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// A timer interface emitter.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TimerInterface {
    /// Interface identifier.
    pub id: String,
    /// Interval schedule in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval_seconds: Option<Number>,
    /// Cron schedule.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cron: Option<String>,
    /// IANA timezone.
    pub timezone: String,
    /// Schedule-change timestamp in epoch milliseconds.
    #[serde(
        deserialize_with = "epoch_millis::deserialize",
        serialize_with = "epoch_millis::serialize"
    )]
    pub schedule_changed_at: DateTime<Utc>,
    /// Creation timestamp in epoch milliseconds.
    #[serde(
        deserialize_with = "epoch_millis::deserialize",
        serialize_with = "epoch_millis::serialize"
    )]
    pub created_at: DateTime<Utc>,
    /// Last update timestamp in epoch milliseconds.
    #[serde(
        deserialize_with = "epoch_millis::deserialize",
        serialize_with = "epoch_millis::serialize"
    )]
    pub updated_at: DateTime<Utc>,
    /// Discriminator.
    #[serde(rename = "type")]
    pub type_name: String,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// A deployed emitter interface.
#[derive(Clone, Debug, PartialEq)]
pub enum Emitter {
    /// A deployed component.
    DeployedComponent(DeployedComponent),
    /// An HTTP interface.
    HttpInterface(HttpInterface),
    /// A timer interface.
    TimerInterface(TimerInterface),
    /// An interface type added by the provider.
    Other(Value),
}

impl Serialize for Emitter {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::DeployedComponent(value) => value.serialize(serializer),
            Self::HttpInterface(value) => value.serialize(serializer),
            Self::TimerInterface(value) => value.serialize(serializer),
            Self::Other(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Emitter {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        let type_name = value.get("type").and_then(Value::as_str);
        match type_name {
            Some("DeployedComponent") => decode(value, Self::DeployedComponent),
            Some("HttpInterface") => decode(value, Self::HttpInterface),
            Some("TimerInterface") => decode(value, Self::TimerInterface),
            _ => Ok(Self::Other(value)),
        }
    }
}

fn decode<T, F, E>(value: Value, constructor: F) -> std::result::Result<Emitter, E>
where
    T: for<'de> Deserialize<'de>,
    F: FnOnce(T) -> Emitter,
    E: serde::de::Error,
{
    match serde_json::from_value(value) {
        Ok(value) => Ok(constructor(value)),
        Err(error) => Err(E::custom(error)),
    }
}

string_enum! {
    /// Emitter family filter for deployed trigger lists.
    pub enum EmitterType {
        /// Email source.
        Email => "email",
        /// HTTP source.
        Http => "http",
        /// Event source.
        Source => "source",
        /// Timer source.
        Timer => "timer",
    }
}
