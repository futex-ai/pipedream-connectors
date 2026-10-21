//! Lossless connection success and error webhook models.

use pipedream_connect_types::{Account, ConnectToken, Environment, UnknownFields};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{PayloadError, check_body_limit};

/// The two connection event discriminator values defined by Connect.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum ConnectionEventKind {
    /// A connected account was created or refreshed successfully.
    #[serde(rename = "CONNECTION_SUCCESS")]
    Success,
    /// The connection flow failed.
    #[serde(rename = "CONNECTION_ERROR")]
    Error,
}

/// A connection webhook payload.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ConnectionEvent {
    /// Event discriminator.
    pub event: ConnectionEventKind,
    /// Short-lived Connect token for this connection flow.
    pub connect_token: ConnectToken,
    /// Project environment that handled the connection.
    pub environment: Environment,
    /// Lossless session identifier, including large JSON integers.
    pub connect_session_id: Value,
    /// Connected account metadata on a successful connection.
    #[serde(default)]
    pub account: Option<Account>,
    /// Provider error message on a failed connection.
    #[serde(default)]
    pub error: Option<String>,
    /// Unknown top-level fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

impl ConnectionEvent {
    /// Decodes a connection event without applying a body limit.
    pub fn decode(raw_body: &[u8]) -> Result<Self, PayloadError> {
        Self::decode_with_limit(raw_body, usize::MAX)
    }

    /// Checks the body limit before decoding a connection event.
    pub fn decode_with_limit(raw_body: &[u8], body_limit: usize) -> Result<Self, PayloadError> {
        check_body_limit(raw_body, body_limit)?;
        let event: Self = match serde_json::from_slice(raw_body) {
            Ok(event) => event,
            Err(_) => return Err(PayloadError::InvalidJson),
        };
        event.validate()
    }

    fn validate(self) -> Result<Self, PayloadError> {
        if self.connect_session_id.is_null() {
            return Err(PayloadError::InvalidConnectionEvent);
        }
        if self
            .account
            .as_ref()
            .is_some_and(|account| account.credentials.is_some())
        {
            return Err(PayloadError::CredentialsPresent);
        }
        match &self.event {
            ConnectionEventKind::Success if self.account.is_none() => {
                Err(PayloadError::InvalidConnectionEvent)
            }
            ConnectionEventKind::Error if self.error.is_none() => {
                Err(PayloadError::InvalidConnectionEvent)
            }
            _ => Ok(self),
        }
    }
}

#[cfg(test)]
#[path = "_tests_/events_tests.rs"]
mod events_tests;
