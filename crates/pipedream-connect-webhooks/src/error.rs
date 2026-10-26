//! Errors returned while decoding bounded webhook payloads.

use thiserror::Error;

/// Identifies whether the sender or receiving host caused an error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FaultClass {
    /// The request sender supplied invalid or oversized input.
    Sender,
    /// The receiving host supplied invalid configuration or state.
    Host,
}

impl FaultClass {
    pub(crate) const fn status_code(self, sender_status: u16) -> u16 {
        match self {
            Self::Sender => sender_status,
            Self::Host => 500,
        }
    }
}

/// The default maximum raw webhook body size, in bytes.
pub const DEFAULT_BODY_LIMIT_BYTES: usize = 10 * 1024 * 1024;

/// Errors returned while decoding a connection event or trigger delivery.
#[derive(Debug, Error)]
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
    /// A raw JSON value could not be parsed or decoded as the requested type.
    #[error("[pipedream-connect-webhooks/payload] JSON value conversion failed")]
    Json {
        /// The original JSON parsing or decoding error.
        #[source]
        source: serde_json::Error,
    },
    /// The JSON does not satisfy the connection event contract.
    #[error("[pipedream-connect-webhooks/payload] invalid connection event")]
    InvalidConnectionEvent,
    /// The JSON does not satisfy the trigger delivery contract.
    #[error("[pipedream-connect-webhooks/payload] invalid trigger delivery")]
    InvalidTriggerDelivery,
    /// The trigger context identifier is empty, too long, or contains a control character.
    #[error("[pipedream-connect-webhooks/payload] invalid trigger context id")]
    InvalidContextId,
}

impl PayloadError {
    /// Returns the sender or host class for this error.
    pub const fn fault_class(&self) -> FaultClass {
        match self {
            Self::BodyTooLarge { .. }
            | Self::InvalidJson
            | Self::Json { .. }
            | Self::InvalidConnectionEvent
            | Self::InvalidTriggerDelivery
            | Self::InvalidContextId => FaultClass::Sender,
        }
    }

    /// Returns the HTTP status that normally represents this payload failure.
    pub const fn status_code(&self) -> u16 {
        let sender_status = match self {
            Self::BodyTooLarge { .. } => 413,
            Self::InvalidJson
            | Self::Json { .. }
            | Self::InvalidConnectionEvent
            | Self::InvalidTriggerDelivery
            | Self::InvalidContextId => 400,
        };
        self.fault_class().status_code(sender_status)
    }
}

pub(crate) fn check_body_limit(body: &[u8], limit: usize) -> Result<(), PayloadError> {
    if body.len() > limit {
        Err(PayloadError::BodyTooLarge { limit })
    } else {
        Ok(())
    }
}
