//! Connected account operations and tenant-scoped account cleanup.

use std::sync::Arc;

use async_trait::async_trait;
use pipedream_connect_types::{
    Account, AccountId, AccountRef, AppSlug, CreateAccountRequest, GetAccountRequest,
    ListAccountsRequest, Page,
};

use crate::client::RequestDispatcher;
use crate::error::{Error, Result, WireFailure};

use super::{auth_operations, operation};

/// Connected account operations, consumed through a shared trait object.
#[cfg_attr(
    any(test, feature = "unimock"),
    unimock::unimock(api = AccountsApiMock)
)]
#[async_trait]
pub trait AccountsApi: Send + Sync {
    /// Lists accounts with cursor, app, OAuth app, and credential filters.
    async fn list(&self, request: ListAccountsRequest) -> Result<Page<Account>>;
    /// Retrieves one account and checks its returned identifier.
    async fn get(&self, account_id: AccountId, request: GetAccountRequest) -> Result<Account>;
    /// Retrieves one account and checks its external-user ownership.
    async fn get_owned(&self, account: AccountRef, request: GetAccountRequest) -> Result<Account>;
    /// Creates or reconnects an account from a Connect token.
    async fn create(&self, request: CreateAccountRequest) -> Result<Account>;
    /// Deletes one account by its project-scoped identifier.
    async fn delete(&self, account_id: AccountId) -> Result<()>;
    /// Deletes one account after checking its external-user ownership.
    async fn delete_owned(&self, account: AccountRef) -> Result<()>;
    /// Deletes every account for one app in the project environment.
    async fn delete_by_app(&self, app: AppSlug) -> Result<()>;
}

/// A connected account handle backed by the shared authenticated dispatcher.
#[derive(Clone)]
pub struct Accounts {
    dispatcher: Arc<dyn RequestDispatcher + Send + Sync>,
}

impl Accounts {
    /// Injects shared dispatch without sending a request.
    pub fn new(dispatcher: Arc<dyn RequestDispatcher + Send + Sync>) -> Self {
        Self { dispatcher }
    }
}

#[async_trait]
impl AccountsApi for Accounts {
    async fn list(&self, mut request: ListAccountsRequest) -> Result<Page<Account>> {
        request.include_credentials.get_or_insert(false);
        let page: Page<Account> =
            operation::json(&self.dispatcher, auth_operations::list_accounts(&request)).await?;
        if let Some(external_user_id) = request.external_user_id {
            for account in &page.data {
                if account.external_id.as_ref() != Some(&external_user_id) {
                    return Err(Error::internal(WireFailure::IdentityMismatch));
                }
            }
        }
        Ok(page)
    }

    async fn get(&self, account_id: AccountId, mut request: GetAccountRequest) -> Result<Account> {
        request.include_credentials.get_or_insert(false);
        let response = operation::json(
            &self.dispatcher,
            auth_operations::get_account(&account_id, &request),
        )
        .await?;
        auth_operations::check_account(response, &account_id)
    }

    async fn create(&self, request: CreateAccountRequest) -> Result<Account> {
        operation::json(&self.dispatcher, auth_operations::create_account(&request)).await
    }

    async fn get_owned(
        &self,
        account: AccountRef,
        mut request: GetAccountRequest,
    ) -> Result<Account> {
        request.include_credentials.get_or_insert(false);
        let response = operation::json(
            &self.dispatcher,
            auth_operations::get_owned_account(&account, &request),
        )
        .await?;
        auth_operations::check_account_owner(response, &account)
    }

    async fn delete(&self, account_id: AccountId) -> Result<()> {
        self.dispatcher
            .dispatch(auth_operations::delete_account(&account_id))
            .await
            .map(|_| ())
    }

    async fn delete_owned(&self, account: AccountRef) -> Result<()> {
        self.get_owned(account.clone(), GetAccountRequest::default())
            .await?;
        self.dispatcher
            .dispatch(auth_operations::delete_owned_account(&account)?)
            .await
            .map(|_| ())
    }

    async fn delete_by_app(&self, app: AppSlug) -> Result<()> {
        self.dispatcher
            .dispatch(auth_operations::delete_accounts_by_app(&app))
            .await
            .map(|_| ())
    }
}
