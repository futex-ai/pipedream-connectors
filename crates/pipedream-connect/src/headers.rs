//! Shared JSON and authentication headers without credential-bearing errors.

use http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use http::{HeaderMap, HeaderValue};
use pipedream_connect_types::{Environment, SecretString};

use crate::config::ClientConfig;
use crate::error::{Error, InvalidRequest, Result};

pub(crate) fn json_headers(body: bool) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(concat!("pipedream-connect/", env!("CARGO_PKG_VERSION"))),
    );
    if body {
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    }
    headers
}

pub(crate) fn authenticated(
    config: &ClientConfig,
    token: &SecretString,
    mut headers: HeaderMap,
    json: bool,
    body: bool,
    rate_limit: bool,
) -> Result<HeaderMap> {
    let mut authorization = header_value(&format!("Bearer {}", token.expose_secret()))?;
    authorization.set_sensitive(true);
    headers.insert(AUTHORIZATION, authorization);
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(concat!("pipedream-connect/", env!("CARGO_PKG_VERSION"))),
    );
    let environment = match config.environment {
        Environment::Development => "development",
        Environment::Production => "production",
        Environment::Other(_) => return Err(Error::InvalidRequest(InvalidRequest::Environment)),
    };
    headers.insert("x-pd-environment", HeaderValue::from_static(environment));
    if json {
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    }
    if body {
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    }
    headers.remove("x-pd-rate-limit");
    if rate_limit && let Some(token) = &config.rate_limit_token {
        let mut value = header_value(token.expose_secret())?;
        value.set_sensitive(true);
        headers.insert("x-pd-rate-limit", value);
    }
    Ok(headers)
}

fn header_value(value: &str) -> Result<HeaderValue> {
    match HeaderValue::from_str(value) {
        Ok(value) => Ok(value),
        Err(_) => Err(Error::InvalidRequest(InvalidRequest::Header)),
    }
}
