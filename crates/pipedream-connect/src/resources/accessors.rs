//! Client composition of small, independently mockable resource groups.

use std::sync::Arc;

use crate::client::PipedreamClient;

use super::actions::{Actions, ActionsApi};
use super::app_categories::{AppCategories, AppCategoriesApi};
use super::app_overrides::{AppOverrides, AppOverridesApi};
use super::apps::{Apps, AppsApi};
use super::components::{Components, ComponentsApi};
use super::deployed_triggers::{DeployedTriggers, DeployedTriggersApi};
use super::file_stash::{FileStash, FileStashApi};
use super::oauth_apps::{OauthApps, OauthAppsApi};
use super::project_webhooks::{ProjectWebhooks, ProjectWebhooksApi};
use super::projects::{Projects, ProjectsApi};
use super::rate_limits::{RateLimits, RateLimitsApi};
use super::triggers::{Triggers, TriggersApi};
use super::usage::{Usage, UsageApi};
use super::workflows::{Workflows, WorkflowsApi};

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
    /// Returns deployed trigger lifecycle operations.
    fn deployed_triggers(&self) -> Arc<dyn DeployedTriggersApi + Send + Sync>;
    /// Returns project administration operations.
    fn projects(&self) -> Arc<dyn ProjectsApi + Send + Sync>;
    /// Returns custom OAuth app administration operations.
    fn oauth_apps(&self) -> Arc<dyn OauthAppsApi + Send + Sync>;
    /// Returns app override administration operations.
    fn app_overrides(&self) -> Arc<dyn AppOverridesApi + Send + Sync>;
    /// Returns project environment webhook operations.
    fn project_webhooks(&self) -> Arc<dyn ProjectWebhooksApi + Send + Sync>;
    /// Returns Connect usage operations.
    fn usage(&self) -> Arc<dyn UsageApi + Send + Sync>;
    /// Returns custom rate-limit operations.
    fn rate_limits(&self) -> Arc<dyn RateLimitsApi + Send + Sync>;
    /// Returns File Stash operations.
    fn file_stash(&self) -> Arc<dyn FileStashApi + Send + Sync>;
    /// Returns workflow invocation operations.
    fn workflows(&self) -> Arc<dyn WorkflowsApi + Send + Sync>;
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

    /// Returns deployed trigger lifecycle operations.
    pub fn deployed_triggers(&self) -> Arc<dyn DeployedTriggersApi + Send + Sync> {
        Arc::new(DeployedTriggers::new(self.dispatcher()))
    }

    /// Returns project administration operations.
    pub fn projects(&self) -> Arc<dyn ProjectsApi + Send + Sync> {
        Arc::new(Projects::new(self.dispatcher()))
    }

    /// Returns custom OAuth app administration operations.
    pub fn oauth_apps(&self) -> Arc<dyn OauthAppsApi + Send + Sync> {
        Arc::new(OauthApps::new(self.dispatcher()))
    }

    /// Returns app override administration operations.
    pub fn app_overrides(&self) -> Arc<dyn AppOverridesApi + Send + Sync> {
        Arc::new(AppOverrides::new(self.dispatcher()))
    }

    /// Returns project environment webhook operations.
    pub fn project_webhooks(&self) -> Arc<dyn ProjectWebhooksApi + Send + Sync> {
        Arc::new(ProjectWebhooks::new(self.dispatcher()))
    }

    /// Returns Connect usage operations.
    pub fn usage(&self) -> Arc<dyn UsageApi + Send + Sync> {
        Arc::new(Usage::new(self.dispatcher()))
    }

    /// Returns custom rate-limit operations.
    pub fn rate_limits(&self) -> Arc<dyn RateLimitsApi + Send + Sync> {
        Arc::new(RateLimits::new(self.dispatcher()))
    }

    /// Returns File Stash operations.
    pub fn file_stash(&self) -> Arc<dyn FileStashApi + Send + Sync> {
        Arc::new(FileStash::new(self.dispatcher()))
    }

    /// Returns workflow invocation operations.
    pub fn workflows(&self) -> Arc<dyn WorkflowsApi + Send + Sync> {
        Arc::new(Workflows::new(self.dispatcher()))
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

    fn deployed_triggers(&self) -> Arc<dyn DeployedTriggersApi + Send + Sync> {
        PipedreamClient::deployed_triggers(self)
    }

    fn projects(&self) -> Arc<dyn ProjectsApi + Send + Sync> {
        PipedreamClient::projects(self)
    }

    fn oauth_apps(&self) -> Arc<dyn OauthAppsApi + Send + Sync> {
        PipedreamClient::oauth_apps(self)
    }

    fn app_overrides(&self) -> Arc<dyn AppOverridesApi + Send + Sync> {
        PipedreamClient::app_overrides(self)
    }

    fn project_webhooks(&self) -> Arc<dyn ProjectWebhooksApi + Send + Sync> {
        PipedreamClient::project_webhooks(self)
    }

    fn usage(&self) -> Arc<dyn UsageApi + Send + Sync> {
        PipedreamClient::usage(self)
    }

    fn rate_limits(&self) -> Arc<dyn RateLimitsApi + Send + Sync> {
        PipedreamClient::rate_limits(self)
    }

    fn file_stash(&self) -> Arc<dyn FileStashApi + Send + Sync> {
        PipedreamClient::file_stash(self)
    }

    fn workflows(&self) -> Arc<dyn WorkflowsApi + Send + Sync> {
        PipedreamClient::workflows(self)
    }
}
