//! Connect token creation and validation operations.

use std::sync::Arc;

use async_trait::async_trait;
use pipedream_connect_types::{
    ConnectToken, ConnectTokenResponse, CreateConnectTokenRequest, ValidateTokenRequest,
    ValidateTokenResponse,
};

use crate::client::RequestDispatcher;
use crate::error::Result;

use super::{auth_operations, operation};

/// Managed Connect token operations, consumed through a shared trait object.
#[cfg_attr(
    any(test, feature = "unimock"),
    unimock::unimock(api = TokensApiMock)
)]
#[async_trait]
pub trait TokensApi: Send + Sync {
    /// Creates a short-lived Connect token and hosted Connect Link URL.
    async fn create(&self, request: CreateConnectTokenRequest) -> Result<ConnectTokenResponse>;
    /// Validates a Connect token for an app and optional account or OAuth app.
    async fn validate(
        &self,
        token: ConnectToken,
        request: ValidateTokenRequest,
    ) -> Result<ValidateTokenResponse>;
}

/// A Connect token handle backed by the shared authenticated dispatcher.
#[derive(Clone)]
pub struct Tokens {
    dispatcher: Arc<dyn RequestDispatcher + Send + Sync>,
}

impl Tokens {
    /// Injects shared dispatch without sending a request.
    pub fn new(dispatcher: Arc<dyn RequestDispatcher + Send + Sync>) -> Self {
        Self { dispatcher }
    }
}

#[async_trait]
impl TokensApi for Tokens {
    async fn create(&self, request: CreateConnectTokenRequest) -> Result<ConnectTokenResponse> {
        operation::json(&self.dispatcher, auth_operations::create_token(&request)).await
    }

    async fn validate(
        &self,
        token: ConnectToken,
        request: ValidateTokenRequest,
    ) -> Result<ValidateTokenResponse> {
        operation::json(
            &self.dispatcher,
            auth_operations::validate_token(token.expose_secret(), &request),
        )
        .await
    }
}
