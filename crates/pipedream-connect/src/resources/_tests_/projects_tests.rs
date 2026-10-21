//! Project administration request and identity tests.

use pipedream_connect_types::{
    CreateProjectRequest, ListProjectsRequest, ProjectId, UpdateProjectLogoRequest,
    UpdateProjectRequest,
};
use serde_json::json;

use crate::error::Error;
use crate::tests::support::{Harness, response, static_config};

#[tokio::test]
async fn project_operations_use_the_documented_routes_and_bodies() {
    let harness = Harness::script(
        static_config(),
        vec![
            Ok(response(200, json!({"data":[],"page_info":{"count":0}}))),
            Ok(response(201, json!({"id":"proj_New123","name":"New"}))),
            Ok(response(200, json!({"id":"proj_Test123","name":"Current"}))),
            Ok(response(200, json!({"id":"proj_Test123","name":"Updated"}))),
            Ok(response(204, json!(null))),
            Ok(response(200, json!(null))),
            Ok(response(200, json!({"apps":[]}))),
        ],
    );
    harness
        .client
        .projects()
        .list(ListProjectsRequest::default())
        .await
        .unwrap();
    assert!(
        harness
            .request(0)
            .url
            .expose_secret()
            .ends_with("/v1/connect/projects?limit=100")
    );
    harness
        .client
        .projects()
        .create(CreateProjectRequest {
            name: "New".into(),
            app_name: None,
            support_email: None,
            connect_require_key_auth_test: None,
        })
        .await
        .unwrap();
    assert_eq!(harness.request(1).method, http::Method::POST);
    assert_eq!(
        harness.request(1).url.expose_secret(),
        "https://api.pipedream.com/v1/connect/projects"
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(harness.request(1).body.as_ref().unwrap())
            .unwrap()["name"],
        "New"
    );
    let project_id = ProjectId::new("proj_Test123").unwrap();
    harness
        .client
        .projects()
        .get(project_id.clone())
        .await
        .unwrap();
    harness
        .client
        .projects()
        .update(
            project_id.clone(),
            UpdateProjectRequest {
                name: Some("Updated".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    harness
        .client
        .projects()
        .delete(project_id.clone())
        .await
        .unwrap();
    harness
        .client
        .projects()
        .update_logo(
            project_id,
            UpdateProjectLogoRequest {
                logo: "data:image/png;base64,AA==".into(),
            },
        )
        .await
        .unwrap();
    harness.client.projects().info().await.unwrap();
    assert_eq!(
        harness.request(6).url.expose_secret(),
        "https://api.pipedream.com/v1/connect/proj_Test123/projects/info"
    );
    harness.assert_requests(7);
}

#[tokio::test]
async fn project_get_rejects_a_response_for_another_project() {
    let harness = Harness::script(
        static_config(),
        vec![Ok(response(200, json!({"id":"proj_Other","name":"Other"})))],
    );
    let error = harness
        .client
        .projects()
        .get(ProjectId::new("proj_Test123").unwrap())
        .await
        .unwrap_err();
    assert!(matches!(error, Error::Internal(_)));
    harness.assert_requests(1);
}
