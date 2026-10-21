//! Client composition of small, independently mockable resource groups.

use std::sync::Arc;

use crate::client::PipedreamClient;

use super::accounts::{Accounts, AccountsApi};
use super::actions::{Actions, ActionsApi};
use super::app_categories::{AppCategories, AppCategoriesApi};
use super::apps::{Apps, AppsApi};
use super::components::{Components, ComponentsApi};
use super::proxy::{Proxy, ProxyApi};
use super::tokens::{Tokens, TokensApi};
use super::triggers::{Triggers, TriggersApi};
use super::users::{Users, UsersApi};

/// Factory for all resource groups backed by one Connect client.
#[cfg_attr(
    any(test, feature = "unimock"),
    unimock::unimock(api = ConnectResourcesMock)
)]
pub trait ConnectResources: Send + Sync {
    /// Returns app discovery operations.
    fn apps(&self) -> Arc<dyn AppsApi + Send + Sync>;
    /// Returns app category operations.
    fn app_categories(&self) -> Arc<dyn AppCategoriesApi + Send + Sync>;
    /// Returns generic registry component operations.
    fn components(&self) -> Arc<dyn ComponentsApi + Send + Sync>;
    /// Returns registry action operations.
    fn actions(&self) -> Arc<dyn ActionsApi + Send + Sync>;
    /// Returns registry trigger operations.
    fn triggers(&self) -> Arc<dyn TriggersApi + Send + Sync>;
    /// Returns managed Connect token operations.
    fn tokens(&self) -> Arc<dyn TokensApi + Send + Sync>;
    /// Returns connected account operations.
    fn accounts(&self) -> Arc<dyn AccountsApi + Send + Sync>;
    /// Returns external-user operations.
    fn users(&self) -> Arc<dyn UsersApi + Send + Sync>;
    /// Returns authenticated proxy operations.
    fn proxy(&self) -> Arc<dyn ProxyApi + Send + Sync>;
}

impl PipedreamClient {
    /// Returns app discovery operations sharing this client's token cache.
    pub fn apps(&self) -> Arc<dyn AppsApi + Send + Sync> {
        Arc::new(Apps::new(self.dispatcher()))
    }

    /// Returns app category operations sharing this client's token cache.
    pub fn app_categories(&self) -> Arc<dyn AppCategoriesApi + Send + Sync> {
        Arc::new(AppCategories::new(self.dispatcher()))
    }

    /// Returns generic registry component operations.
    pub fn components(&self) -> Arc<dyn ComponentsApi + Send + Sync> {
        Arc::new(Components::new(self.dispatcher()))
    }

    /// Returns registry action discovery, configuration, and execution operations.
    pub fn actions(&self) -> Arc<dyn ActionsApi + Send + Sync> {
        Arc::new(Actions::new(self.dispatcher()))
    }

    /// Returns registry trigger discovery and configuration operations.
    pub fn triggers(&self) -> Arc<dyn TriggersApi + Send + Sync> {
        Arc::new(Triggers::new(self.dispatcher()))
    }

    /// Returns managed Connect token operations sharing this client's token cache.
    pub fn tokens(&self) -> Arc<dyn TokensApi + Send + Sync> {
        Arc::new(Tokens::new(self.dispatcher()))
    }

    /// Returns connected account operations sharing this client's token cache.
    pub fn accounts(&self) -> Arc<dyn AccountsApi + Send + Sync> {
        Arc::new(Accounts::new(self.dispatcher()))
    }

    /// Returns external-user operations sharing this client's token cache.
    pub fn users(&self) -> Arc<dyn UsersApi + Send + Sync> {
        Arc::new(Users::new(self.dispatcher()))
    }

    /// Returns authenticated proxy operations sharing this client's token cache.
    pub fn proxy(&self) -> Arc<dyn ProxyApi + Send + Sync> {
        Arc::new(Proxy::new(self.dispatcher()))
    }
}

impl ConnectResources for PipedreamClient {
    fn apps(&self) -> Arc<dyn AppsApi + Send + Sync> {
        PipedreamClient::apps(self)
    }

    fn app_categories(&self) -> Arc<dyn AppCategoriesApi + Send + Sync> {
        PipedreamClient::app_categories(self)
    }

    fn components(&self) -> Arc<dyn ComponentsApi + Send + Sync> {
        PipedreamClient::components(self)
    }

    fn actions(&self) -> Arc<dyn ActionsApi + Send + Sync> {
        PipedreamClient::actions(self)
    }

    fn triggers(&self) -> Arc<dyn TriggersApi + Send + Sync> {
        PipedreamClient::triggers(self)
    }

    fn tokens(&self) -> Arc<dyn TokensApi + Send + Sync> {
        PipedreamClient::tokens(self)
    }

    fn accounts(&self) -> Arc<dyn AccountsApi + Send + Sync> {
        PipedreamClient::accounts(self)
    }

    fn users(&self) -> Arc<dyn UsersApi + Send + Sync> {
        PipedreamClient::users(self)
    }

    fn proxy(&self) -> Arc<dyn ProxyApi + Send + Sync> {
        PipedreamClient::proxy(self)
    }
}
