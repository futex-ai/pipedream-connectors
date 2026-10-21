//! Immutable client configuration and redacted authentication values.

use std::time::Duration;

use http::HeaderValue;
use pipedream_connect_types::{Environment, ProjectId, SecretString};
use url::Url;

use crate::error::{Error, InvalidRequest, Result};
use crate::retry::RetryPolicy;

/// OAuth credentials used only at the token endpoint.
#[derive(Clone, Debug)]
pub struct ClientCredentials {
    pub(crate) client_id: SecretString,
    pub(crate) client_secret: SecretString,
}

impl ClientCredentials {
    /// Validates credentials without exposing them in an error.
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>) -> Result<Self> {
        let client_id = client_id.into();
        let client_secret = client_secret.into();
        if [&client_id, &client_secret].iter().any(|value| {
            value.is_empty() || value.trim() != *value || value.chars().any(char::is_control)
        }) {
            return Err(Error::InvalidRequest(InvalidRequest::Credentials));
        }
        Ok(Self {
            client_id: SecretString::new(client_id),
            client_secret: SecretString::new(client_secret),
        })
    }
}

/// Authentication selected by the host.
#[derive(Clone, Debug)]
pub enum Authentication {
    /// Acquire and cache OAuth client-credentials bearers.
    ClientCredentials(ClientCredentials),
    /// Use a host-managed bearer without a token request.
    StaticBearer(SecretString),
}

/// Configurable HTTP time and response bounds.
#[derive(Clone, Debug)]
pub struct HttpOptions {
    /// Time allowed to establish a connection.
    pub connect_timeout: Duration,
    /// Total time allowed for one request, including its response body.
    pub request_timeout: Duration,
    /// Maximum response body length in bytes.
    pub response_limit: usize,
}

impl Default for HttpOptions {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(60),
            response_limit: 10 * 1024 * 1024,
        }
    }
}

/// Validated settings shared by authentication and Connect dispatch.
#[derive(Clone, Debug)]
pub struct ClientConfig {
    pub(crate) origin: Url,
    pub(crate) project_id: ProjectId,
    pub(crate) environment: Environment,
    pub(crate) authentication: Authentication,
    pub(crate) scope: Option<String>,
    pub(crate) rate_limit_token: Option<SecretString>,
    pub(crate) http: HttpOptions,
    pub(crate) retry: Option<RetryPolicy>,
}

impl ClientConfig {
    /// Selects a project, explicit environment, and authentication.
    pub fn new(
        project_id: ProjectId,
        environment: Environment,
        authentication: Authentication,
    ) -> Result<Self> {
        if !matches!(
            environment,
            Environment::Development | Environment::Production
        ) {
            return Err(Error::InvalidRequest(InvalidRequest::Environment));
        }
        if let Authentication::StaticBearer(token) = &authentication {
            validate_bearer(token)?;
        }
        Ok(Self {
            origin: parse_origin("https://api.pipedream.com")?,
            project_id,
            environment,
            authentication,
            scope: None,
            rate_limit_token: None,
            http: HttpOptions::default(),
            retry: None,
        })
    }

    /// Selects a bare HTTPS origin without path, query, or credentials.
    pub fn with_origin(mut self, origin: &str) -> Result<Self> {
        self.origin = parse_origin(origin)?;
        Ok(self)
    }

    /// Selects the optional space-separated OAuth scopes.
    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// Selects a custom rate-limit token sent only on Connect requests.
    pub fn with_rate_limit_token(mut self, token: SecretString) -> Result<Self> {
        if HeaderValue::from_str(token.expose_secret()).is_err() {
            return Err(Error::InvalidRequest(InvalidRequest::Header));
        }
        self.rate_limit_token = Some(token);
        Ok(self)
    }

    /// Selects the HTTP time and response bounds.
    pub fn with_http_options(mut self, options: HttpOptions) -> Self {
        self.http = options;
        self
    }

    /// Enables GET retries with an explicit attempt and delay budget.
    pub fn with_retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry = Some(policy);
        self
    }

    /// Returns the HTTP time and response bounds.
    pub fn http_options(&self) -> &HttpOptions {
        &self.http
    }

    /// Returns the validated project identifier.
    pub fn project_id(&self) -> &ProjectId {
        &self.project_id
    }

    /// Returns the explicit request environment.
    pub fn environment(&self) -> &Environment {
        &self.environment
    }
}

pub(crate) fn validate_bearer(token: &SecretString) -> Result<()> {
    if token.expose_secret().is_empty()
        || !token
            .expose_secret()
            .bytes()
            .all(|byte| byte.is_ascii_graphic())
        || HeaderValue::from_str(token.expose_secret()).is_err()
    {
        return Err(Error::InvalidRequest(InvalidRequest::Bearer));
    }
    Ok(())
}

fn parse_origin(value: &str) -> Result<Url> {
    let invalid = || Error::InvalidRequest(InvalidRequest::Origin);
    let url = match Url::parse(value) {
        Ok(url) => url,
        Err(_) => return Err(invalid()),
    };
    let bare = value.split_once("://").is_some_and(|(scheme, rest)| {
        let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
        scheme.eq_ignore_ascii_case("https") && !authority.contains('@') && path.is_empty()
    });
    if !bare
        || value.trim() != value
        || value.chars().any(char::is_control)
        || value.contains('\\')
        || url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(invalid());
    }
    Ok(url)
}
