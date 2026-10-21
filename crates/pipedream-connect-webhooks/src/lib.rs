//! Signature verification and event models for Pipedream Connect webhooks.

#![warn(unreachable_pub)]

mod error;
mod events;
mod signature;
mod trigger;

pub use error::{DecodeError, PayloadError};
pub use events::{ConnectionEvent, ConnectionEventKind};
#[cfg(feature = "unimock")]
pub use signature::WebhookSignatureVerifierMock;
pub use signature::{
    DEFAULT_SIGNATURE_TOLERANCE_SECONDS, DynWebhookSignatureVerifier, HmacWebhookSignatureVerifier,
    SignatureError, SignatureResult, WebhookSignatureVerifier,
};
pub use trigger::{TriggerContext, TriggerDelivery};
