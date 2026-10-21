//! Pure operation descriptions and a single percent-encoding URL builder.

use std::fmt;

use http::{HeaderMap, Method};
use pipedream_connect_types::PageRequest;
use serde::Serialize;
use url::Url;

use crate::config::ClientConfig;
use crate::error::{Error, Result, WireFailure, validate};
use crate::json::encode;
use crate::validation::Validate;

/// The documented route root for an API operation.
#[derive(Clone)]
#[non_exhaustive]
pub enum ApiPath {
    /// `/v1/connect/{project_id}` followed by individual path segments.
    Project(Vec<String>),
    /// `/v1/connect` followed by individual path segments.
    Connect(Vec<String>),
    /// `/v1` followed by individual path segments.
    V1(Vec<String>),
}

impl ApiPath {
    /// Describes a project-scoped route without joining untrusted segments.
    pub fn project(segments: &[&str]) -> Self {
        Self::Project(segments.iter().map(|value| (*value).to_owned()).collect())
    }

    /// Describes a Connect route without a project prefix.
    pub fn connect(segments: &[&str]) -> Self {
        Self::Connect(segments.iter().map(|value| (*value).to_owned()).collect())
    }
}

impl fmt::Debug for ApiPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ApiPath([REDACTED])")
    }
}

/// Operation-specific treatment of successful responses.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum ResponseMode {
    /// JSON operation with normal status mapping.
    #[default]
    Json,
    /// Execution operation with structured errors and an optional 202 result.
    Execution {
        /// Whether 202 means accepted without waiting for completion.
        nonblocking: bool,
    },
    /// Byte operation with normal status mapping.
    Bytes,
    /// Return a 3xx response to the caller without following it.
    Redirect,
    /// Preserve every status, header, and byte after execution-error checks.
    ///
    /// Never retries. A 401 still invalidates its bearer without replay.
    PassThrough,
}

/// A validated operation ready for authenticated dispatch.
#[derive(Clone)]
pub struct ApiRequest {
    pub(crate) method: Method,
    pub(crate) path: ApiPath,
    pub(crate) query: Vec<(String, String)>,
    pub(crate) headers: HeaderMap,
    pub(crate) body: Option<Vec<u8>>,
    pub(crate) json_body: bool,
    pub(crate) mode: ResponseMode,
}

impl ApiRequest {
    /// Creates an operation with no body, query, or caller headers.
    pub fn new(method: Method, path: ApiPath) -> Self {
        Self {
            method,
            path,
            query: Vec::new(),
            headers: HeaderMap::new(),
            body: None,
            json_body: false,
            mode: ResponseMode::Json,
        }
    }

    /// Adds a query pair. Repeated pairs preserve array filters.
    pub fn query_pair(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.query.push((name.into(), value.into()));
        self
    }

    /// Validates and appends the cursor pagination query.
    pub fn with_page(mut self, page: &PageRequest) -> Result<Self> {
        validate(page.validate())?;
        self.query
            .push(("limit".to_owned(), page.limit.to_string()));
        if let Some(cursor) = &page.after {
            self.query
                .push(("after".to_owned(), cursor.as_str().to_owned()));
        }
        if let Some(cursor) = &page.before {
            self.query
                .push(("before".to_owned(), cursor.as_str().to_owned()));
        }
        Ok(self)
    }

    /// Validates a request model before serializing its JSON body.
    ///
    /// ```compile_fail
    /// use http::Method;
    /// use pipedream_connect::request::{ApiPath, ApiRequest};
    ///
    /// let unvalidated = serde_json::json!({"expires_in": 0});
    /// ApiRequest::new(Method::POST, ApiPath::project(&["tokens"]))
    ///     .with_json(&unvalidated);
    /// ```
    pub fn with_json<T: Serialize + Validate>(mut self, model: &T) -> Result<Self> {
        model.validate()?;
        self.body = Some(encode(model)?);
        self.json_body = true;
        Ok(self)
    }

    /// Selects an exact byte body with content headers supplied by the caller.
    pub fn with_bytes(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self.json_body = false;
        self
    }

    /// Adds typed HTTP headers. Dispatch replaces SDK-owned headers.
    pub fn with_headers(mut self, headers: HeaderMap) -> Self {
        self.headers = headers;
        self
    }

    /// Selects the documented response contract for this operation.
    pub fn with_response_mode(mut self, mode: ResponseMode) -> Self {
        self.mode = mode;
        self
    }
}

impl fmt::Debug for ApiRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApiRequest")
            .field("method", &self.method)
            .field("path", &"[REDACTED]")
            .field("query", &"[REDACTED]")
            .field("headers", &"[REDACTED]")
            .field("body_bytes", &self.body.as_ref().map(Vec::len))
            .field("mode", &self.mode)
            .finish()
    }
}

pub(crate) fn build_url(
    config: &ClientConfig,
    path: &ApiPath,
    query: &[(String, String)],
) -> Result<Url> {
    let mut url = config.origin.clone();
    let (prefix, segments) = match path {
        ApiPath::Project(segments) => (vec!["v1", "connect", config.project_id.as_str()], segments),
        ApiPath::Connect(segments) => (vec!["v1", "connect"], segments),
        ApiPath::V1(segments) => (vec!["v1"], segments),
    };
    match url.path_segments_mut() {
        Ok(mut path) => {
            path.clear().extend(prefix);
            path.extend(segments.iter().map(String::as_str));
        }
        Err(()) => return Err(Error::internal(WireFailure::Url)),
    }
    if !query.is_empty() {
        url.query_pairs_mut().extend_pairs(query);
    }
    Ok(url)
}
