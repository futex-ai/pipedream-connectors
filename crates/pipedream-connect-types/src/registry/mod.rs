//! Registry applications, components, annotations, and list requests.

mod enums;
pub use enums::*;

use serde::{Deserialize, Serialize};

use crate::{AppCategoryId, AppSlug, ComponentKey, PageRequest, UnknownFields};

/// Named OAuth scope profile returned by an app.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ScopeProfile {
    /// Profile name.
    pub name: ScopeProfileName,
    /// Upstream OAuth scopes.
    pub scopes: Vec<String>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Proxy capabilities published by an app.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ProxySupport {
    /// Whether proxy calls are enabled.
    #[serde(default)]
    pub proxy_enabled: bool,
    /// Provider target domains accepted by the proxy.
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    /// Base URL for app-relative proxy targets.
    #[serde(default)]
    pub base_proxy_target_url: Option<String>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Public Pipedream app metadata.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct App {
    /// OAuth app identifier, when the app is custom.
    #[serde(default)]
    pub id: Option<String>,
    /// Provider app slug.
    pub name_slug: AppSlug,
    /// Human-readable app name.
    pub name: String,
    /// Authentication mode.
    #[serde(default)]
    pub auth_type: Option<AppAuthType>,
    /// Public app description.
    #[serde(default)]
    pub description: Option<String>,
    /// App logo URL.
    pub img_src: String,
    /// JSON-encoded custom field metadata.
    #[serde(default)]
    pub custom_fields_json: Option<String>,
    /// App categories.
    #[serde(default)]
    pub categories: Vec<String>,
    /// Provider popularity ordering.
    #[serde(default)]
    pub featured_weight: f64,
    /// Named OAuth scope profiles.
    #[serde(default)]
    pub scope_profiles: Vec<ScopeProfile>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// App category metadata.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AppCategory {
    /// Category identifier.
    #[serde(default)]
    pub id: Option<AppCategoryId>,
    /// Human-readable category name.
    #[serde(default)]
    pub name: Option<String>,
    /// Public category description.
    #[serde(default)]
    pub description: Option<String>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Component behavior annotations.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolAnnotations {
    /// Whether the component may destroy external data.
    #[serde(default)]
    pub destructive_hint: Option<bool>,
    /// Whether repeated calls are idempotent.
    #[serde(default)]
    pub idempotent_hint: Option<bool>,
    /// Whether the component interacts with an open world.
    #[serde(default)]
    pub open_world_hint: Option<bool>,
    /// Whether the component only reads.
    #[serde(default)]
    pub read_only_hint: Option<bool>,
    /// Human-readable tool title.
    #[serde(default)]
    pub title: Option<String>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// A registry component and its configurable properties.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Component {
    /// Stable component key.
    pub key: ComponentKey,
    /// Human-readable component name.
    pub name: String,
    /// Semantic component version.
    pub version: String,
    /// Configurable properties.
    pub configurable_props: Vec<crate::ConfigurableProp>,
    /// Public description.
    #[serde(default)]
    pub description: Option<String>,
    /// Component kind.
    #[serde(default)]
    pub component_type: Option<ComponentType>,
    /// File Stash requirement.
    #[serde(default)]
    pub stash: Option<ComponentStash>,
    /// Tool behavior hints.
    #[serde(default)]
    pub annotations: Option<ToolAnnotations>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// App catalog query with bounded cursor pagination.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
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
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
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
