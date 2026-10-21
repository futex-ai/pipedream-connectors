//! Registry models and requests.

use serde::{Deserialize, Serialize};

use crate::{ComponentKey, ComponentStash, ComponentType, ConfigurableProp, UnknownFields};

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
    pub configurable_props: Vec<ConfigurableProp>,
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
