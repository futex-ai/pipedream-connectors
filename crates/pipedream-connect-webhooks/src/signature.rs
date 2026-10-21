//! Exact raw-body HMAC-SHA256 verification for Connect webhook requests.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use pipedream_connect_types::SecretString;
use sha2::Sha256;
use subtle::ConstantTimeEq;
use thiserror::Error;

/// The default absolute timestamp tolerance in seconds.
pub const DEFAULT_SIGNATURE_TOLERANCE_SECONDS: u64 = 300;

/// Errors returned by the webhook signature verifier.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum SignatureError {
    /// The header is not exactly `t=<seconds>,v1=<hex>`.
    #[error("[pipedream-connect-webhooks/signature] invalid signature header")]
    InvalidHeader,
    /// The signing key is empty or cannot initialize HMAC.
    #[error("[pipedream-connect-webhooks/signature] invalid signing key")]
    InvalidKey,
    /// The timestamp is outside the configured two-sided replay window.
    #[error("[pipedream-connect-webhooks/signature] stale signature timestamp")]
    StaleTimestamp,
    /// The computed digest does not match the supplied digest.
    #[error("[pipedream-connect-webhooks/signature] invalid signature")]
    InvalidSignature,
    /// The receipt time is before the Unix epoch.
    #[error("[pipedream-connect-webhooks/signature] invalid receipt time")]
    InvalidReceiptTime,
    /// The raw body is larger than the limit supplied to the verifier.
    #[error("[pipedream-connect-webhooks/signature] body exceeds the configured limit")]
    BodyTooLarge {
        /// The configured maximum body size in bytes.
        limit: usize,
    },
}

/// The result type returned by signature verification.
pub type SignatureResult<T> = std::result::Result<T, SignatureError>;

/// Injectable signature verification capability for webhook receivers.
#[cfg_attr(feature = "unimock", unimock::unimock(api = WebhookSignatureVerifierMock))]
pub trait WebhookSignatureVerifier: Send + Sync {
    /// Verifies a signed raw body against the trusted receipt time.
    fn verify(
        &self,
        key: &SecretString,
        signature_header: &str,
        raw_body: &[u8],
        receipt_time: SystemTime,
    ) -> SignatureResult<()>;
}

/// A shared signature verifier trait object.
pub type DynWebhookSignatureVerifier = Arc<dyn WebhookSignatureVerifier + Send + Sync>;

/// HMAC-SHA256 verifier with an explicit raw-body limit and replay tolerance.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HmacWebhookSignatureVerifier {
    body_limit: usize,
    tolerance_seconds: u64,
}

impl HmacWebhookSignatureVerifier {
    /// Constructs a verifier with the caller's body limit and the 300-second default tolerance.
    pub const fn new(body_limit: usize) -> Self {
        Self {
            body_limit,
            tolerance_seconds: DEFAULT_SIGNATURE_TOLERANCE_SECONDS,
        }
    }

    /// Sets the absolute replay tolerance in seconds.
    pub const fn with_tolerance_seconds(mut self, tolerance_seconds: u64) -> Self {
        self.tolerance_seconds = tolerance_seconds;
        self
    }

    /// Returns the configured body limit in bytes.
    pub const fn body_limit(self) -> usize {
        self.body_limit
    }

    /// Returns the configured absolute replay tolerance in seconds.
    pub const fn tolerance_seconds(self) -> u64 {
        self.tolerance_seconds
    }

    fn verify_inner(
        &self,
        key: &SecretString,
        signature_header: &str,
        raw_body: &[u8],
        receipt_time: SystemTime,
    ) -> SignatureResult<()> {
        if raw_body.len() > self.body_limit {
            return Err(SignatureError::BodyTooLarge {
                limit: self.body_limit,
            });
        }
        if key.expose_secret().is_empty() {
            return Err(SignatureError::InvalidKey);
        }
        let (timestamp_text, timestamp, provided) = parse_signature_header(signature_header)?;
        let receipt_seconds = match receipt_time.duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_secs(),
            Err(_) => return Err(SignatureError::InvalidReceiptTime),
        };
        if receipt_seconds.abs_diff(timestamp) > self.tolerance_seconds {
            return Err(SignatureError::StaleTimestamp);
        }
        let mut mac = match <Hmac<Sha256> as hmac::digest::KeyInit>::new_from_slice(
            key.expose_secret().as_bytes(),
        ) {
            Ok(mac) => mac,
            Err(_) => return Err(SignatureError::InvalidKey),
        };
        mac.update(timestamp_text.as_bytes());
        mac.update(b".");
        mac.update(raw_body);
        let computed = mac.finalize().into_bytes();
        if computed.ct_eq(&provided).unwrap_u8() != 1 {
            return Err(SignatureError::InvalidSignature);
        }
        Ok(())
    }
}

impl WebhookSignatureVerifier for HmacWebhookSignatureVerifier {
    fn verify(
        &self,
        key: &SecretString,
        signature_header: &str,
        raw_body: &[u8],
        receipt_time: SystemTime,
    ) -> SignatureResult<()> {
        self.verify_inner(key, signature_header, raw_body, receipt_time)
    }
}

impl SignatureError {
    /// Returns the HTTP status that normally represents this verification failure.
    pub const fn status_code(self) -> u16 {
        match self {
            Self::BodyTooLarge { .. } => 413,
            Self::InvalidSignature | Self::StaleTimestamp | Self::InvalidKey => 401,
            Self::InvalidHeader | Self::InvalidReceiptTime => 400,
        }
    }
}

fn parse_signature_header(header: &str) -> SignatureResult<(&str, u64, [u8; 32])> {
    let mut parts = header.split(',');
    let timestamp_part = parts.next().ok_or(SignatureError::InvalidHeader)?;
    let digest_part = parts.next().ok_or(SignatureError::InvalidHeader)?;
    if parts.next().is_some() {
        return Err(SignatureError::InvalidHeader);
    }
    let (timestamp_name, timestamp_text) = timestamp_part
        .split_once('=')
        .ok_or(SignatureError::InvalidHeader)?;
    let (digest_name, digest_text) = digest_part
        .split_once('=')
        .ok_or(SignatureError::InvalidHeader)?;
    if timestamp_name != "t" || digest_name != "v1" {
        return Err(SignatureError::InvalidHeader);
    }
    if timestamp_text.is_empty() || !timestamp_text.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(SignatureError::InvalidHeader);
    }
    let timestamp = match timestamp_text.parse::<u64>() {
        Ok(timestamp) => timestamp,
        Err(_) => return Err(SignatureError::InvalidHeader),
    };
    if digest_text.len() != 64 {
        return Err(SignatureError::InvalidHeader);
    }
    let mut digest = [0; 32];
    if hex::decode_to_slice(digest_text, &mut digest).is_err() {
        return Err(SignatureError::InvalidHeader);
    }
    Ok((timestamp_text, timestamp, digest))
}

#[cfg(test)]
#[path = "_tests_/signature_tests.rs"]
mod signature_tests;
