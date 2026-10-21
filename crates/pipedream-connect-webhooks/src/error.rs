//! Errors returned while decoding bounded webhook payloads.

use thiserror::Error;

/// Errors returned while decoding a connection event or trigger delivery.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum PayloadError {
    /// The raw body is larger than the limit supplied by the caller.
    #[error("[pipedream-connect-webhooks/payload] body exceeds the configured limit")]
    BodyTooLarge {
        /// The configured maximum body size in bytes.
        limit: usize,
    },
    /// The body is not valid JSON.
    #[error("[pipedream-connect-webhooks/payload] invalid JSON body")]
    InvalidJson,
    /// The JSON does not satisfy the connection event contract.
    #[error("[pipedream-connect-webhooks/payload] invalid connection event")]
    InvalidConnectionEvent,
    /// The JSON does not satisfy the trigger delivery contract.
    #[error("[pipedream-connect-webhooks/payload] invalid trigger delivery")]
    InvalidTriggerDelivery,
    /// The trigger context identifier is empty, too long, or contains a control character.
    #[error("[pipedream-connect-webhooks/payload] invalid trigger context id")]
    InvalidContextId,
    /// A connection event contained credential data in its account object.
    #[error("[pipedream-connect-webhooks/payload] connection account contains credentials")]
    CredentialsPresent,
}

/// Alias for callers that use the term decode error.
pub type DecodeError = PayloadError;

impl PayloadError {
    /// Returns the HTTP status that normally represents this payload failure.
    pub const fn status_code(self) -> u16 {
        match self {
            Self::BodyTooLarge { .. } => 413,
            Self::InvalidJson
            | Self::InvalidConnectionEvent
            | Self::InvalidTriggerDelivery
            | Self::InvalidContextId
            | Self::CredentialsPresent => 400,
        }
    }
}

pub(crate) fn check_body_limit(body: &[u8], limit: usize) -> Result<(), PayloadError> {
    if body.len() > limit {
        Err(PayloadError::BodyTooLarge { limit })
    } else {
        Ok(())
    }
}
