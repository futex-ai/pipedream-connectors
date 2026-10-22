//! External-user inventory and cleanup operations.

use std::sync::Arc;

use async_trait::async_trait;
use pipedream_connect_types::{
    Account, ExternalUser, ExternalUserId, ListUserAccountsRequest, ListUsersRequest, Page,
};

use crate::client::RequestDispatcher;
use crate::error::Result;

use super::{auth_operations, operation};

/// External-user operations, consumed through a shared trait object.
#[cfg_attr(
    any(test, feature = "unimock"),
    unimock::unimock(api = UsersApiMock)
)]
#[async_trait]
pub trait UsersApi: Send + Sync {
    /// Lists external users with cursor and partial-match filters.
    async fn list(&self, request: ListUsersRequest) -> Result<Page<ExternalUser>>;
    /// Deletes an external user and the resources owned by that user.
    async fn delete(&self, external_user_id: ExternalUserId) -> Result<()>;
    /// Lists accounts owned by one external user.
    async fn list_accounts(
        &self,
        external_user_id: ExternalUserId,
        request: ListUserAccountsRequest,
    ) -> Result<Vec<Account>>;
}

/// An external-user handle backed by the shared authenticated dispatcher.
#[derive(Clone)]
pub struct Users {
    dispatcher: Arc<dyn RequestDispatcher + Send + Sync>,
}

impl Users {
    /// Injects shared dispatch without sending a request.
    pub fn new(dispatcher: Arc<dyn RequestDispatcher + Send + Sync>) -> Self {
        Self { dispatcher }
    }
}

#[async_trait]
impl UsersApi for Users {
    async fn list(&self, request: ListUsersRequest) -> Result<Page<ExternalUser>> {
        operation::json(&self.dispatcher, auth_operations::list_users(&request)).await
    }

    async fn delete(&self, external_user_id: ExternalUserId) -> Result<()> {
        operation::no_content(
            &self.dispatcher,
            Ok(auth_operations::delete_user(external_user_id.as_str())),
        )
        .await
    }

    async fn list_accounts(
        &self,
        external_user_id: ExternalUserId,
        mut request: ListUserAccountsRequest,
    ) -> Result<Vec<Account>> {
        request.include_credentials.get_or_insert(false);
        let accounts: Vec<Account> = operation::json(
            &self.dispatcher,
            auth_operations::list_user_accounts(external_user_id.as_str(), &request),
        )
        .await?;
        auth_operations::check_owned_accounts(&accounts, &external_user_id, None)?;
        Ok(accounts)
    }
}

#[cfg(test)]
#[path = "_tests_/users_tests.rs"]
mod users_tests;
