//! Default reqwest and system-clock composition for the public client builder.

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use pipedream_connect_types::{Environment, ProjectId, SecretString};

use crate::client::PipedreamClient;
use crate::config::{Authentication, ClientConfig, ClientCredentials, HttpOptions};
use crate::error::{ConfigurationField, Error, InvalidRequest, Result};
use crate::reqwest_transport::ReqwestTransport;
use crate::retry::RetryPolicy;
use crate::runtime::{SystemClock, TokioSleeper};

/// A builder with no implicit project, environment, or authentication.
#[derive(Default)]
pub struct ClientBuilder {
    project: Option<ProjectId>,
    environment: Option<Environment>,
    authentication: Option<Authentication>,
    origin: Option<String>,
    scope: Option<String>,
    rate_limit_token: Option<SecretString>,
    http: HttpOptions,
    retry: Option<RetryPolicy>,
}

impl PipedreamClient {
    /// Starts a builder that constructs reqwest and the system clock.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }
}

impl ClientBuilder {
    /// Selects the project identifier.
    pub fn project_id(mut self, project: ProjectId) -> Self {
        self.project = Some(project);
        self
    }

    /// Selects the explicit project environment.
    pub fn environment(mut self, environment: Environment) -> Self {
        self.environment = Some(environment);
        self
    }

    /// Selects OAuth client credentials.
    pub fn client_credentials(mut self, credentials: ClientCredentials) -> Self {
        self.authentication = Some(Authentication::ClientCredentials(credentials));
        self
    }

    /// Selects a static bearer without calling the token endpoint.
    pub fn static_bearer(mut self, token: SecretString) -> Self {
        self.authentication = Some(Authentication::StaticBearer(token));
        self
    }

    /// Selects another bare HTTPS origin.
    pub fn origin(mut self, origin: impl Into<String>) -> Self {
        self.origin = Some(origin.into());
        self
    }

    /// Selects the OAuth space-separated scopes.
    pub fn scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// Selects a custom rate-limit token for Connect requests.
    pub fn rate_limit_token(mut self, token: SecretString) -> Self {
        self.rate_limit_token = Some(token);
        self
    }

    /// Selects the connection timeout.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.http.connect_timeout = timeout;
        self
    }

    /// Selects the total request timeout, including response bodies.
    pub fn request_timeout(mut self, timeout: Duration) -> Self {
        self.http.request_timeout = timeout;
        self
    }

    /// Selects the response body byte cap.
    pub fn response_limit(mut self, limit: usize) -> Self {
        self.http.response_limit = limit;
        self
    }

    /// Enables GET retries with caller-selected attempt and delay bounds.
    pub fn retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.retry = Some(policy);
        self
    }

    /// Validates configuration and constructs the client without network calls.
    pub fn build(self) -> Result<PipedreamClient> {
        let mut config = ClientConfig::new(
            self.project
                .ok_or_else(|| missing(ConfigurationField::ProjectId))?,
            self.environment
                .ok_or_else(|| missing(ConfigurationField::Environment))?,
            self.authentication
                .ok_or_else(|| missing(ConfigurationField::Authentication))?,
        )?
        .with_http_options(self.http);
        if let Some(origin) = self.origin {
            config = config.with_origin(&origin)?;
        }
        if let Some(scope) = self.scope {
            config = config.with_scope(scope);
        }
        if let Some(token) = self.rate_limit_token {
            config = config.with_rate_limit_token(token)?;
        }
        if let Some(policy) = self.retry {
            config = config.with_retry_policy(policy);
        }
        let transport = Arc::new(ReqwestTransport::new(config.http_options())?);
        PipedreamClient::new(
            config,
            transport,
            Arc::new(SystemClock::default()),
            Arc::new(TokioSleeper),
        )
    }
}

fn missing(field: ConfigurationField) -> Error {
    Error::InvalidRequest(InvalidRequest::MissingConfiguration { field })
}

impl fmt::Debug for ClientBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientBuilder")
            .field("project", &self.project)
            .field("environment", &self.environment)
            .field("authentication", &"[REDACTED]")
            .field("origin", &"[REDACTED]")
            .field("rate_limit_token", &"[REDACTED]")
            .field("http", &self.http)
            .field("retry", &self.retry)
            .finish_non_exhaustive()
    }
}
