//! Project administration models.

use serde::{Deserialize, Serialize};

use crate::{AppSlug, ProjectId, UnknownFields};

/// A Connect project.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Project {
    /// Project identifier.
    pub id: ProjectId,
    /// Display name.
    pub name: String,
    /// Connect app name.
    #[serde(default)]
    pub app_name: Option<String>,
    /// Support email.
    #[serde(default)]
    pub support_email: Option<String>,
    /// Whether key-based account checks are required.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connect_require_key_auth_test: Option<bool>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Request to create a project.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CreateProjectRequest {
    /// Project display name.
    pub name: String,
    /// Connect app name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
    /// Support email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_email: Option<String>,
    /// Whether key-based account checks are required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_require_key_auth_test: Option<bool>,
}

/// Partial project update.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct UpdateProjectRequest {
    /// New display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// New Connect app name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
    /// New support email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub support_email: Option<String>,
    /// New key-auth test policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_require_key_auth_test: Option<bool>,
}

/// Project logo update body.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct UpdateProjectLogoRequest {
    /// Data URI containing a base64 image.
    pub logo: String,
}

/// One app enabled for a project environment.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ProjectInfoApp {
    /// Provider app identifier, when returned.
    #[serde(default)]
    pub id: Option<String>,
    /// Provider app slug.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name_slug: Option<AppSlug>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}

/// Apps enabled for a project environment.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ProjectInfo {
    /// Enabled apps.
    pub apps: Vec<ProjectInfoApp>,
    /// Unknown provider fields.
    #[serde(flatten)]
    pub extra: UnknownFields,
}
