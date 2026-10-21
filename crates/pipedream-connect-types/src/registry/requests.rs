//! Registry models and requests.

use serde::{Deserialize, Serialize};

use crate::{
    AppCategoryId, AppSlug, AppSortKey, ComponentType, PageRequest, RegistryKind, SortDirection,
};

/// App catalog query with bounded cursor pagination.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ListAppsRequest {
    /// Cursor and page limit.
    #[serde(flatten)]
    pub page: PageRequest,
    /// Full-text query sent as `q`.
    #[serde(rename = "q", skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    /// Listing sort key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_key: Option<AppSortKey>,
    /// Listing direction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_direction: Option<SortDirection>,
    /// Category filters.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub category_ids: Vec<AppCategoryId>,
    /// Require components.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_components: Option<bool>,
    /// Require actions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_actions: Option<bool>,
    /// Require triggers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_triggers: Option<bool>,
}

/// Component catalog query with bounded cursor pagination.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ListComponentsRequest {
    /// Cursor and page limit.
    #[serde(flatten)]
    pub page: PageRequest,
    /// Full-text query sent as `q`.
    #[serde(rename = "q", skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    /// App filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<AppSlug>,
    /// Registry source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry: Option<RegistryKind>,
    /// Component kind filter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_type: Option<ComponentType>,
}

/// Component, action, or registry trigger version query.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct GetComponentRequest {
    /// Semantic version, or the provider default when omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
