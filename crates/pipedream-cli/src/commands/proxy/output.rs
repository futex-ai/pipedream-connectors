//! The lossless JSON form of an upstream proxy response.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use pipedream_connect::resources::proxy::ProxyResponse;
use serde::Serialize;

use crate::redact::Redact;

/// An upstream response: its status, every header, and its body.
#[derive(Debug, Eq, PartialEq, Serialize)]
pub(super) struct ProxyOutput {
    /// The upstream HTTP status.
    status: u16,
    /// Every response header, with repeated names kept.
    headers: Vec<HeaderEntry>,
    /// The response body.
    body: Encoded,
}

/// One response header.
#[derive(Debug, Eq, PartialEq, Serialize)]
struct HeaderEntry {
    /// The lowercase header name.
    name: String,
    /// The value encoding.
    encoding: Encoding,
    /// The value as text or base64.
    value: String,
}

/// Bytes as UTF-8 text when they are valid UTF-8, or else as base64.
#[derive(Debug, Eq, PartialEq, Serialize)]
struct Encoded {
    /// The encoding of `value`.
    encoding: Encoding,
    /// The bytes as text or base64.
    value: String,
}

/// How the output represents bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
enum Encoding {
    /// The bytes are valid UTF-8 text.
    #[serde(rename = "utf-8")]
    Utf8,
    /// The bytes are standard base64 with padding.
    #[serde(rename = "base64")]
    Base64,
}

impl Encoded {
    fn new(bytes: &[u8]) -> Self {
        match std::str::from_utf8(bytes) {
            Ok(text) => Self {
                encoding: Encoding::Utf8,
                value: text.to_owned(),
            },
            Err(_) => Self {
                encoding: Encoding::Base64,
                value: STANDARD.encode(bytes),
            },
        }
    }
}

impl From<ProxyResponse> for ProxyOutput {
    fn from(response: ProxyResponse) -> Self {
        let headers = response
            .headers
            .iter()
            .map(|(name, value)| {
                let encoded = Encoded::new(value.as_bytes());
                HeaderEntry {
                    name: name.as_str().to_owned(),
                    encoding: encoded.encoding,
                    value: encoded.value,
                }
            })
            .collect();
        Self {
            status: response.status.as_u16(),
            headers,
            body: Encoded::new(&response.body),
        }
    }
}

/// Upstream headers and bodies are app data, not credential-class values.
impl Redact for ProxyOutput {
    fn redact(&mut self) {}
}

#[cfg(test)]
#[path = "_tests_/output_tests.rs"]
mod output_tests;
