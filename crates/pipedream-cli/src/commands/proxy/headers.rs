//! `--header` parsing and the Connect proxy header policy.
//!
//! The SDK applies the same policy before the token call, but its error does
//! not name the header. The CLI checks first, so the error can name it.

use http::{HeaderMap, HeaderName, HeaderValue};

use crate::error::{Error, HeaderProblem, Result};

/// Header names that Pipedream reserves and the proxy never forwards.
pub(super) const RESERVED: [&str; 19] = [
    "accept-encoding",
    "access-control-request-headers",
    "access-control-request-method",
    "connection",
    "content-length",
    "cookie",
    "date",
    "dnt",
    "expect",
    "host",
    "keep-alive",
    "origin",
    "permissions-policy",
    "referer",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
    "via",
];

/// Header name prefixes that Pipedream reserves.
pub(super) const RESERVED_PREFIXES: [&str; 3] = ["proxy-", "sec-", "x-pd-"];

/// Parses each `Name: value` argument into a header that the proxy forwards.
///
/// Errors name the header or its position. They never contain a header value.
pub(super) fn parse(arguments: &[String]) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    for (index, argument) in arguments.iter().enumerate() {
        let position = index + 1;
        let invalid = |problem| Error::InvalidHeader { position, problem };
        let Some((name, value)) = argument.split_once(':') else {
            return Err(invalid(HeaderProblem::Format));
        };
        let name = name.trim();
        if name.is_empty() {
            return Err(invalid(HeaderProblem::Format));
        }
        let Ok(name) = HeaderName::from_bytes(name.as_bytes()) else {
            return Err(invalid(HeaderProblem::Name));
        };
        if reserved(name.as_str()) {
            return Err(invalid(HeaderProblem::Reserved {
                name: name.as_str().to_owned(),
            }));
        }
        let Ok(value) = HeaderValue::from_bytes(value.trim().as_bytes()) else {
            return Err(invalid(HeaderProblem::Value {
                name: name.as_str().to_owned(),
            }));
        };
        headers.append(name, value);
    }
    Ok(headers)
}

/// Returns whether Pipedream reserves the lowercase header `name`.
fn reserved(name: &str) -> bool {
    RESERVED.contains(&name)
        || RESERVED_PREFIXES
            .iter()
            .any(|prefix| name.starts_with(prefix))
}

#[cfg(test)]
#[path = "_tests_/headers_tests.rs"]
mod headers_tests;
