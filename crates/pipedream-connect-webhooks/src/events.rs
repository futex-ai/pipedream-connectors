//! Lossless connection success and error webhook models.

use pipedream_connect_types::{Account, ConnectToken, Environment};
use serde::{Deserialize, Deserializer, Serialize};

use crate::error::{DEFAULT_BODY_LIMIT_BYTES, PayloadError};
use crate::raw_json::{RawFields, RawJson, optional, parse_bounded, required};

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
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ConnectionEvent {
    /// Event discriminator.
    pub event: ConnectionEventKind,
    /// Short-lived Connect token for this connection flow.
    pub connect_token: ConnectToken,
    /// Project environment that handled the connection.
    pub environment: Environment,
    /// Lossless session identifier, including large JSON integers.
    pub connect_session_id: RawJson,
    /// Connected account metadata on a successful connection.
    #[serde(default)]
    pub account: Option<Account>,
    /// Provider error message on a failed connection.
    #[serde(default)]
    pub error: Option<String>,
    /// Unknown top-level fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: RawFields,
}

impl<'de> Deserialize<'de> for ConnectionEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawJson::deserialize(deserializer)?;
        match Self::decode_value(&raw) {
            Ok(event) => Ok(event),
            Err(error) => Err(serde::de::Error::custom(error)),
        }
    }
}

impl ConnectionEvent {
    /// Decodes a connection event with the default 10 MiB body limit.
    pub fn decode(raw_body: &[u8]) -> Result<Self, PayloadError> {
        Self::decode_with_limit(raw_body, DEFAULT_BODY_LIMIT_BYTES)
    }

    /// Checks the body limit before decoding a connection event.
    pub fn decode_with_limit(raw_body: &[u8], body_limit: usize) -> Result<Self, PayloadError> {
        let raw = parse_bounded(raw_body, body_limit)?;
        Self::decode_value(&raw)
    }

    fn decode_value(raw: &RawJson) -> Result<Self, PayloadError> {
        let fields: RawFields = match raw.decode() {
            Ok(fields) => fields,
            Err(_) => return Err(PayloadError::InvalidConnectionEvent),
        };
        Self::from_fields(fields)
    }

    fn from_fields(mut fields: RawFields) -> Result<Self, PayloadError> {
        let event = match required::<ConnectionEventKind, serde_json::Error>(&mut fields, "event") {
            Ok(event) => event,
            Err(_) => return Err(PayloadError::InvalidConnectionEvent),
        };
        let connect_token =
            match required::<ConnectToken, serde_json::Error>(&mut fields, "connect_token") {
                Ok(connect_token) => connect_token,
                Err(_) => return Err(PayloadError::InvalidConnectionEvent),
            };
        let environment =
            match required::<Environment, serde_json::Error>(&mut fields, "environment") {
                Ok(environment) => environment,
                Err(_) => return Err(PayloadError::InvalidConnectionEvent),
            };
        let connect_session_id = fields
            .remove("connect_session_id")
            .ok_or(PayloadError::InvalidConnectionEvent)?;
        let account = match optional::<Account, serde_json::Error>(&mut fields, "account") {
            Ok(account) => account,
            Err(_) => return Err(PayloadError::InvalidConnectionEvent),
        };
        let error = match optional::<String, serde_json::Error>(&mut fields, "error") {
            Ok(error) => error,
            Err(_) => return Err(PayloadError::InvalidConnectionEvent),
        };
        let event = Self {
            event,
            connect_token,
            environment,
            connect_session_id,
            account,
            error,
            extra: fields,
        };
        event.validate()
    }

    fn validate(self) -> Result<Self, PayloadError> {
        if self.connect_session_id.as_str().trim() == "null" {
            return Err(PayloadError::InvalidConnectionEvent);
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
