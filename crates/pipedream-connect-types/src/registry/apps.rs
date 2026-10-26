//! Registry models and requests.

use serde::{Deserialize, Serialize};
use serde_json::Number;

use crate::{AppAuthType, AppCategoryId, AppSlug, ScopeProfileName, UnknownFields};

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
    pub proxy_enabled: Option<bool>,
    /// Provider target domains accepted by the proxy.
    #[serde(default)]
    pub allowed_domains: Option<Vec<String>>,
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
    /// Provider app identifier, when returned.
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<String>>,
    /// Provider popularity ordering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub featured_weight: Option<Number>,
    /// Named OAuth scope profiles.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_profiles: Option<Vec<ScopeProfile>>,
    /// App proxy capabilities, when returned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connect: Option<ProxySupport>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// App category metadata.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AppCategory {
    /// Category identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<AppCategoryId>,
    /// Human-readable category name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Public category description.
    #[serde(default)]
    pub description: Option<String>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}
