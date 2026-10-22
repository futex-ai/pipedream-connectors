//! Project administration and enabled-app information operations.

use std::sync::Arc;

use async_trait::async_trait;
use pipedream_connect_types::{
    CreateProjectRequest, ListProjectsRequest, Page, Project, ProjectId, ProjectInfo,
    UpdateProjectLogoRequest, UpdateProjectRequest,
};

use crate::client::RequestDispatcher;
use crate::error::Result;

use super::admin_operations;
use super::operation;

/// Project administration operations, consumed through a shared trait object.
#[cfg_attr(
    any(test, feature = "unimock"),
    unimock::unimock(api = ProjectsApiMock)
)]
#[async_trait]
pub trait ProjectsApi: Send + Sync {
    /// Lists projects visible to the authenticated owner.
    async fn list(&self, request: ListProjectsRequest) -> Result<Page<Project>>;
    /// Creates a project.
    async fn create(&self, request: CreateProjectRequest) -> Result<Project>;
    /// Retrieves a project and verifies its returned identifier.
    async fn get(&self, project_id: ProjectId) -> Result<Project>;
    /// Updates a project and returns the provider response unchanged.
    async fn update(&self, project_id: ProjectId, request: UpdateProjectRequest)
    -> Result<Project>;
    /// Deletes a project.
    async fn delete(&self, project_id: ProjectId) -> Result<()>;
    /// Updates a project's logo.
    async fn update_logo(
        &self,
        project_id: ProjectId,
        request: UpdateProjectLogoRequest,
    ) -> Result<()>;
    /// Returns apps enabled in the configured project environment.
    async fn info(&self) -> Result<ProjectInfo>;
}

/// A project administration handle backed by the shared dispatcher.
#[derive(Clone)]
pub struct Projects {
    dispatcher: Arc<dyn RequestDispatcher + Send + Sync>,
}

impl Projects {
    /// Injects shared dispatch without sending a request.
    pub fn new(dispatcher: Arc<dyn RequestDispatcher + Send + Sync>) -> Self {
        Self { dispatcher }
    }
}

#[async_trait]
impl ProjectsApi for Projects {
    async fn list(&self, request: ListProjectsRequest) -> Result<Page<Project>> {
        operation::json(&self.dispatcher, admin_operations::list_projects(&request)).await
    }

    async fn create(&self, request: CreateProjectRequest) -> Result<Project> {
        operation::json(&self.dispatcher, admin_operations::create_project(&request)).await
    }

    async fn get(&self, project_id: ProjectId) -> Result<Project> {
        let project = operation::json(
            &self.dispatcher,
            Ok(admin_operations::get_project(&project_id)),
        )
        .await?;
        admin_operations::check_project(project, &project_id)
    }

    async fn update(
        &self,
        project_id: ProjectId,
        request: UpdateProjectRequest,
    ) -> Result<Project> {
        operation::json(
            &self.dispatcher,
            admin_operations::update_project(&project_id, &request),
        )
        .await
    }

    async fn delete(&self, project_id: ProjectId) -> Result<()> {
        operation::no_content(
            &self.dispatcher,
            Ok(admin_operations::delete_project(&project_id)),
        )
        .await
    }

    async fn update_logo(
        &self,
        project_id: ProjectId,
        request: UpdateProjectLogoRequest,
    ) -> Result<()> {
        operation::no_content(
            &self.dispatcher,
            admin_operations::update_project_logo(&project_id, &request),
        )
        .await
    }

    async fn info(&self) -> Result<ProjectInfo> {
        operation::json(&self.dispatcher, Ok(admin_operations::project_info())).await
    }
}

#[cfg(test)]
#[path = "_tests_/projects_tests.rs"]
mod projects_tests;
