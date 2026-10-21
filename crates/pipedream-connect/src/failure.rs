//! Status mapping and rate-limit parsing without storing raw error bodies.

use std::time::{Duration, SystemTime};

use http::HeaderMap;
use pipedream_connect_types::{ErrorResponse, ExecutionError};
use serde_json::Value;

use crate::error::{Error, Result};
use crate::request::ResponseMode;
use crate::transport::HttpResponse;

pub(crate) fn ensure_status(
    response: &HttpResponse,
    mode: ResponseMode,
    now: SystemTime,
) -> Result<()> {
    classify(response, mode, now, |body| {
        serde_json::from_slice::<ErrorResponse>(body).ok()
    })
}

fn classify(
    response: &HttpResponse,
    mode: ResponseMode,
    now: SystemTime,
    decode_error: impl FnOnce(&[u8]) -> Option<ErrorResponse>,
) -> Result<()> {
    let status = response.status.as_u16();
    if mode == (ResponseMode::Delete { missing_ok: true }) && status == 404 {
        return Ok(());
    }
    if matches!(
        mode,
        ResponseMode::Execution { .. } | ResponseMode::PassThrough
    ) {
        if let Some(error) = execution_error(&response.body) {
            return Err(Error::ExecutionFailed {
                status,
                error: Box::new(error),
            });
        }
        if status == 202 && mode == (ResponseMode::Execution { nonblocking: true }) {
            return Err(Error::Accepted);
        }
    }
    if mode == ResponseMode::PassThrough
        || response.status.is_success()
        || mode == ResponseMode::Redirect && response.status.is_redirection()
    {
        return Ok(());
    }
    match status {
        429 => {
            return Err(Error::RateLimited {
                retry_after: retry_after(&response.headers, now),
                limit: number(&response.headers, "x-ratelimit-limit"),
                remaining: number(&response.headers, "x-ratelimit-remaining"),
                reset: number(&response.headers, "x-ratelimit-reset"),
            });
        }
        408 | 500..=599 => {
            return Err(Error::Unavailable {
                status: Some(status),
            });
        }
        _ => {}
    }
    let envelope = decode_error(&response.body);
    let message = envelope.as_ref().and_then(provider_message);
    Err(match status {
        401 => Error::Unauthorized { message },
        403 => Error::Forbidden { message },
        404 => Error::NotFound { message },
        _ => Error::Rejected {
            status,
            code: envelope.and_then(|value| value.code),
            message,
        },
    })
}

fn execution_error(body: &[u8]) -> Option<ExecutionError> {
    let value: Value = serde_json::from_slice(body).ok()?;
    let error = value.get("error")?.as_object()?;
    if !error.contains_key("attribution") && !error.contains_key("name") {
        return None;
    }
    serde_json::from_value(Value::Object(error.clone())).ok()
}

#[cfg(test)]
#[path = "_tests_/status_decode_tests.rs"]
mod status_decode_tests;

/// Isolates the error text representation while model review changes it.
fn provider_message(envelope: &ErrorResponse) -> Option<String> {
    serde_json::to_value(&envelope.error)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
}

pub(crate) fn retry_after(headers: &HeaderMap, now: SystemTime) -> Option<Duration> {
    let value = headers.get("retry-after")?.to_str().ok()?.trim();
    if !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()) {
        return value.parse::<u64>().ok().map(Duration::from_secs);
    }
    let date = httpdate::parse_http_date(value).ok()?;
    Some(date.duration_since(now).unwrap_or(Duration::ZERO))
}

fn number(headers: &HeaderMap, name: &str) -> Option<u64> {
    let value = headers.get(name)?.to_str().ok()?.trim();
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    value.parse().ok()
}

pub(crate) fn check_limit(response: &HttpResponse, limit: usize) -> Result<()> {
    let announced = response
        .headers
        .get("content-length")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok());
    if response.body.len() > limit || announced.is_some_and(|length| length > limit as u64) {
        return Err(Error::ResponseTooLarge { limit });
    }
    Ok(())
}
