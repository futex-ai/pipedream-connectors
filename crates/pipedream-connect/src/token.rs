//! Single-flight OAuth refresh and static bearer providers.

use std::fmt;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use http::Method;
use pipedream_connect_types::{OAuthTokenResponse, SecretString};
use serde::Serialize;
use tokio::sync::Mutex;

use crate::config::{Authentication, ClientConfig, ClientCredentials, validate_bearer};
use crate::error::{Error, InvalidRequest, Result, WireFailure};
use crate::failure::{check_limit, ensure_status};
use crate::headers::json_headers;
use crate::json::encode;
use crate::request::{ApiPath, ResponseMode, build_url};
use crate::runtime::Clock;
use crate::transport::{HttpRequest, HttpTransport};

/// Supplies a bearer and invalidates only the bearer rejected by a request.
#[cfg_attr(any(test, feature = "unimock"), unimock::unimock(api = AccessTokenProviderMock))]
#[async_trait]
pub trait AccessTokenProvider: Send + Sync {
    /// Returns an access token, refreshing it when needed.
    async fn access_token(&self) -> Result<SecretString>;
    /// Invalidates the rejected bearer without replaying its request.
    async fn invalidate(&self, rejected: &SecretString);
}

/// A bearer whose lifecycle belongs to the host, with no token endpoint calls.
#[derive(Debug)]
pub struct StaticTokenProvider {
    token: SecretString,
}

impl StaticTokenProvider {
    /// Validates a host-managed bearer without sending a request.
    pub fn new(token: SecretString) -> Result<Self> {
        validate_bearer(&token)?;
        Ok(Self { token })
    }
}

#[async_trait]
impl AccessTokenProvider for StaticTokenProvider {
    async fn access_token(&self) -> Result<SecretString> {
        Ok(self.token.clone())
    }

    async fn invalidate(&self, _rejected: &SecretString) {}
}

/// OAuth cache whose refresh mutex is held until the bearer is available.
pub struct ClientCredentialsProvider {
    config: ClientConfig,
    credentials: ClientCredentials,
    transport: Arc<dyn HttpTransport + Send + Sync>,
    clock: Arc<dyn Clock + Send + Sync>,
    cached: Mutex<Option<CachedToken>>,
}

struct CachedToken {
    token: SecretString,
    refresh_at: SystemTime,
}

#[derive(Serialize)]
struct TokenRequest<'a> {
    grant_type: &'static str,
    client_id: &'a SecretString,
    client_secret: &'a SecretString,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<&'a str>,
}

impl ClientCredentialsProvider {
    /// Builds an OAuth provider from client-credentials configuration.
    pub fn new(
        config: ClientConfig,
        transport: Arc<dyn HttpTransport + Send + Sync>,
        clock: Arc<dyn Clock + Send + Sync>,
    ) -> Result<Self> {
        let credentials = match &config.authentication {
            Authentication::ClientCredentials(credentials) => credentials.clone(),
            Authentication::StaticBearer(_) => {
                return Err(Error::InvalidRequest(InvalidRequest::Credentials));
            }
        };
        Ok(Self {
            config,
            credentials,
            transport,
            clock,
            cached: Mutex::new(None),
        })
    }
}

#[async_trait]
impl AccessTokenProvider for ClientCredentialsProvider {
    async fn access_token(&self) -> Result<SecretString> {
        let mut cached = self.cached.lock().await;
        let now = self.clock.now();
        if let Some(value) = cached.as_ref()
            && now < value.refresh_at
        {
            return Ok(value.token.clone());
        }
        tracing::debug!("refreshing Pipedream OAuth bearer");
        let body = encode(&TokenRequest {
            grant_type: "client_credentials",
            client_id: &self.credentials.client_id,
            client_secret: &self.credentials.client_secret,
            scope: self.config.scope.as_deref(),
        })?;
        let response = self
            .transport
            .send(HttpRequest {
                method: Method::POST,
                url: SecretString::new(
                    build_url(
                        &self.config,
                        &ApiPath::V1(vec!["oauth".to_owned(), "token".to_owned()]),
                        &[],
                    )?
                    .as_str(),
                ),
                headers: json_headers(true),
                body: Some(body),
                response_limit: self.config.http.response_limit,
                timeout: self.config.http.request_timeout,
            })
            .await?;
        check_limit(&response, self.config.http.response_limit)?;
        ensure_status(&response, ResponseMode::Json, self.clock.now())?;
        let value: OAuthTokenResponse = response.json()?;
        if value.expires_in == 0
            || !value.token_type.eq_ignore_ascii_case("bearer")
            || validate_bearer(&value.access_token).is_err()
        {
            return Err(Error::internal(WireFailure::Token));
        }
        let lifetime = Duration::from_secs(u64::from(value.expires_in));
        let skew = (lifetime / 10).min(Duration::from_secs(30));
        let refresh_at = now
            .checked_add(lifetime - skew)
            .ok_or_else(|| Error::internal(WireFailure::Token))?;
        if refresh_at <= self.clock.now() {
            return Err(Error::Unavailable { status: None });
        }
        let token = value.access_token;
        *cached = Some(CachedToken {
            token: token.clone(),
            refresh_at,
        });
        Ok(token)
    }

    async fn invalidate(&self, rejected: &SecretString) {
        let mut cached = self.cached.lock().await;
        if cached
            .as_ref()
            .is_some_and(|value| &value.token == rejected)
        {
            *cached = None;
        }
    }
}

impl fmt::Debug for ClientCredentialsProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientCredentialsProvider")
            .field("credentials", &"[REDACTED]")
            .field("cached", &"[REDACTED]")
            .finish_non_exhaustive()
    }
}
