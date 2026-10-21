//! Project and trigger webhook models.

use serde::{Deserialize, Serialize};

use crate::{Environment, SecretString, UnknownFields, WebhookId};

/// Project environment webhook metadata.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ProjectWebhook {
    /// Webhook identifier.
    pub id: WebhookId,
    /// Destination URL.
    pub url: String,
    /// Whether a signing key exists.
    pub signing_key_set: bool,
    /// Signing key when returned.
    #[serde(default)]
    pub signing_key: Option<SecretString>,
    /// Creation timestamp in epoch seconds.
    #[serde(default)]
    pub created_at: Option<i64>,
    /// Update timestamp in epoch seconds.
    #[serde(default)]
    pub updated_at: Option<i64>,
    /// Project environment.
    #[serde(default)]
    pub environment: Option<Environment>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// A project or trigger webhook without a response envelope.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Webhook {
    /// Webhook identifier.
    pub id: WebhookId,
    /// Destination URL.
    pub url: String,
    /// Whether a signing key is configured.
    pub signing_key_set: bool,
    /// Creation timestamp in epoch seconds.
    pub created_at: i64,
    /// Last update timestamp in epoch seconds.
    pub updated_at: i64,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// A webhook response that may include a signing key.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct WebhookWithOptionalSigningKey {
    /// Webhook identifier.
    pub id: WebhookId,
    /// Destination URL.
    pub url: String,
    /// Signing key when returned.
    #[serde(default)]
    pub signing_key: Option<SecretString>,
    /// Whether a signing key is configured.
    pub signing_key_set: bool,
    /// Creation timestamp in epoch seconds.
    pub created_at: i64,
    /// Last update timestamp in epoch seconds.
    pub updated_at: i64,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// A webhook response that includes its signing key.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct WebhookWithSigningKey {
    /// Webhook identifier.
    pub id: WebhookId,
    /// Destination URL.
    pub url: String,
    /// Signing key.
    pub signing_key: SecretString,
    /// Whether a signing key is configured.
    pub signing_key_set: bool,
    /// Creation timestamp in epoch seconds.
    pub created_at: i64,
    /// Last update timestamp in epoch seconds.
    pub updated_at: i64,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Project environment webhook destination update.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SetWebhookRequest {
    /// Destination URL.
    pub url: String,
}
