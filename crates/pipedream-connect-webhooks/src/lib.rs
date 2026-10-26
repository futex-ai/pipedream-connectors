#![doc = include_str!("../README.md")]
#![warn(unreachable_pub)]
#![cfg_attr(test, allow(clippy::disallowed_methods))]

mod error;
mod events;
mod raw_json;
mod signature;
mod trigger;

pub use error::{DEFAULT_BODY_LIMIT_BYTES, FaultClass, PayloadError};
pub use events::{ConnectionEvent, ConnectionEventKind};
pub use raw_json::{RawFields, RawJson};
#[cfg(feature = "unimock")]
pub use signature::WebhookSignatureVerifierMock;
pub use signature::{
    DEFAULT_SIGNATURE_TOLERANCE_SECONDS, DynWebhookSignatureVerifier, HmacWebhookSignatureVerifier,
    SignatureError, SignatureResult, WebhookSignatureVerifier,
};
pub use trigger::{TriggerContext, TriggerDelivery};
