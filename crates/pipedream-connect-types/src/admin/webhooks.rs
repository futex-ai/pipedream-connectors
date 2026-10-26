//! Project and trigger webhook models.

use serde::{Deserialize, Serialize};

use crate::{SecretString, Timestamp, UnknownFields, WebhookId};

/// A project or trigger webhook without a response envelope.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Webhook {
    /// Webhook identifier.
    pub id: WebhookId,
    /// Destination URL.
    pub url: String,
    /// Whether a signing key is configured.
    pub signing_key_set: bool,
    /// Creation timestamp.
    pub created_at: Timestamp,
    /// Last update timestamp.
    pub updated_at: Timestamp,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signing_key: Option<SecretString>,
    /// Whether a signing key is configured.
    pub signing_key_set: bool,
    /// Creation timestamp.
    pub created_at: Timestamp,
    /// Last update timestamp.
    pub updated_at: Timestamp,
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
    /// Creation timestamp.
    pub created_at: Timestamp,
    /// Last update timestamp.
    pub updated_at: Timestamp,
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
