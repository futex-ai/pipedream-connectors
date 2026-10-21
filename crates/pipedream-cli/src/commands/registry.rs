//! Registry filters shared by the component and action list commands.

use clap::{Args, ValueEnum};
use pipedream_connect_types::{AppSlug, ComponentType, RegistryKind};

use crate::argument::Argument;
use crate::error::{Result, checked};

/// The filters of the registry list commands.
#[derive(Debug, Args)]
pub(crate) struct RegistryArgs {
    /// Search text.
    #[arg(long, value_name = "TEXT")]
    query: Option<String>,
    /// App slug filter, such as google_calendar.
    #[arg(long, value_name = "APP")]
    app: Option<String>,
    /// Registry source.
    #[arg(long, value_enum)]
    registry: Option<RegistryArg>,
}

/// Validated registry filters.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct RegistryFilters {
    /// Search text.
    pub(crate) query: Option<String>,
    /// App filter.
    pub(crate) app: Option<AppSlug>,
    /// Registry source.
    pub(crate) registry: Option<RegistryKind>,
}

impl RegistryArgs {
    /// Validates the filters. Sends no request.
    pub(crate) fn filters(self) -> Result<RegistryFilters> {
        Ok(RegistryFilters {
            query: self.query,
            app: self
                .app
                .map(|app| checked(AppSlug::new(app), Argument::AppFilter))
                .transpose()?,
            registry: self.registry.map(RegistryArg::model),
        })
    }
}

/// A registry source.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum RegistryArg {
    /// Public registry components.
    Public,
    /// Private registry components.
    Private,
    /// Public and private registry components.
    All,
}

impl RegistryArg {
    fn model(self) -> RegistryKind {
        match self {
            Self::Public => RegistryKind::Public,
            Self::Private => RegistryKind::Private,
            Self::All => RegistryKind::All,
        }
    }
}

/// A registry component kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum ComponentTypeArg {
    /// Actions.
    Action,
    /// Triggers.
    Trigger,
}

impl ComponentTypeArg {
    /// Returns the SDK component type for this value.
    pub(crate) fn model(self) -> ComponentType {
        match self {
            Self::Action => ComponentType::Action,
            Self::Trigger => ComponentType::Trigger,
        }
    }
}
