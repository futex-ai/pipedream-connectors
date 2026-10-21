//! Client composition of small, independently mockable resource groups.

use std::sync::Arc;

use crate::client::PipedreamClient;

use super::actions::{Actions, ActionsApi};
use super::app_categories::{AppCategories, AppCategoriesApi};
use super::apps::{Apps, AppsApi};
use super::components::{Components, ComponentsApi};
use super::triggers::{Triggers, TriggersApi};

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
}
