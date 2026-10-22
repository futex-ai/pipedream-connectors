use super::*;

pub async fn invoke(client: &PipedreamClient, id: &str) -> bool {
    let user = || ExternalUserId::new("customer_1").expect("external user ID");
    let trigger = || DeployedTriggerId::new("dc_Trigger123").expect("trigger ID");
    let query = || DeployedTriggerQuery {
        external_user_id: user(),
    };
    match id {
        "listDeployedTriggers" => {
            let _ = client
                .deployed_triggers()
                .list(ListDeployedTriggersRequest {
                    external_user_id: user(),
                    page: PageRequest::default(),
                    emitter_type: None,
                })
                .await;
        }
        "retrieveDeployedTrigger" => {
            let _ = client.deployed_triggers().get(trigger(), query()).await;
        }
        "updateDeployedTrigger" => {
            let _ = client
                .deployed_triggers()
                .update(trigger(), query(), UpdateTriggerRequest::default())
                .await;
        }
        "deleteDeployedTrigger" => {
            let _ = client
                .deployed_triggers()
                .delete(
                    trigger(),
                    DeleteTriggerRequest {
                        external_user_id: user(),
                        ignore_hook_errors: None,
                        missing_ok: false,
                    },
                )
                .await;
        }
        "listDeployedTriggerEvents" => {
            let request = ListTriggerEventsRequest::new(user(), 10).unwrap();
            let _ = client
                .deployed_triggers()
                .list_events(trigger(), request)
                .await;
        }
        "listDeployedTriggerWorkflows" => {
            let _ = client
                .deployed_triggers()
                .get_workflows(trigger(), query())
                .await;
        }
        "updateDeployedTriggerWorkflows" => {
            let request = UpdateTriggerWorkflowsRequest {
                workflow_ids: vec![WorkflowEndpoint::new("p_workflow1").unwrap()],
            };
            let _ = client
                .deployed_triggers()
                .set_workflows(trigger(), query(), request)
                .await;
        }
        "listDeployedTriggerWebhooks" => {
            let _ = client
                .deployed_triggers()
                .list_webhooks(trigger(), query())
                .await;
        }
        "updateDeployedTriggerWebhooks" => {
            let request = UpdateTriggerWebhooksRequest {
                webhook_urls: vec!["https://example.com/events".into()],
            };
            let _ = client
                .deployed_triggers()
                .set_webhooks(trigger(), query(), request)
                .await;
        }
        "retrieveDeployedTriggerWebhook" => {
            let _ = client
                .deployed_triggers()
                .get_webhook(trigger(), WebhookId::new("wh_Webhook123").unwrap(), query())
                .await;
        }
        "regenerateDeployedTriggerWebhookSigningKey" => {
            let _ = client
                .deployed_triggers()
                .regenerate_webhook_signing_key(
                    trigger(),
                    WebhookId::new("wh_Webhook123").unwrap(),
                    query(),
                )
                .await;
        }
        "listOauthApps" => {
            let _ = client.oauth_apps().list(Default::default()).await;
        }
        "createOauthApp" => {
            let _ = client
                .oauth_apps()
                .create(CreateOauthAppRequest {
                    app: "google_calendar".into(),
                    client_id: "upstream-client".into(),
                    client_secret: SecretString::new("upstream-secret"),
                    name: None,
                    description: None,
                    scopes: None,
                    additional_scopes: None,
                })
                .await;
        }
        "retrieveOauthApp" => {
            let _ = client
                .oauth_apps()
                .get(OauthAppId::new("oa_Test123").unwrap())
                .await;
        }
        "updateOauthApp" => {
            let _ = client
                .oauth_apps()
                .update(
                    OauthAppId::new("oa_Test123").unwrap(),
                    UpdateOauthAppRequest::default(),
                )
                .await;
        }
        "deleteOauthApp" => {
            let _ = client
                .oauth_apps()
                .delete(OauthAppId::new("oa_Test123").unwrap())
                .await;
        }
        "retrieveProjectEnvironmentWebhook" => {
            let _ = client.project_webhooks().get().await;
        }
        "updateProjectEnvironmentWebhook" => {
            let _ = client
                .project_webhooks()
                .set(SetWebhookRequest {
                    url: "https://example.com/events".into(),
                })
                .await;
        }
        "deleteProjectEnvironmentWebhook" => {
            let _ = client.project_webhooks().delete().await;
        }
        "regenerateProjectEnvironmentWebhookSigningKey" => {
            let _ = client.project_webhooks().regenerate_signing_key().await;
        }
        "listProjects" => {
            let _ = client.projects().list(Default::default()).await;
        }
        "createProject" => {
            let _ = client
                .projects()
                .create(CreateProjectRequest {
                    name: "Demo".into(),
                    app_name: None,
                    support_email: None,
                    connect_require_key_auth_test: None,
                })
                .await;
        }
        "retrieveProject" => {
            let _ = client.projects().get(project_id()).await;
        }
        "deleteProject" => {
            let _ = client.projects().delete(project_id()).await;
        }
        "updateProject" => {
            let _ = client
                .projects()
                .update(
                    project_id(),
                    UpdateProjectRequest {
                        name: Some("Updated".into()),
                        ..Default::default()
                    },
                )
                .await;
        }
        "updateProjectLogo" => {
            let _ = client
                .projects()
                .update_logo(
                    project_id(),
                    UpdateProjectLogoRequest {
                        logo: "data:image/png;base64,AA==".into(),
                    },
                )
                .await;
        }
        "retrieveProjectInfo" => {
            let _ = client.projects().info().await;
        }
        "downloadFileStashFile" => {
            let _ = client
                .file_stash()
                .download(DownloadFileStashRequest {
                    s3_key: "stash-key".into(),
                })
                .await;
        }
        "retrieveProxy" | "postProxy" | "putProxy" | "deleteProxy" | "patchProxy" => {
            let _ = client.proxy().send(proxy_request(id)).await;
        }
        "createToken" => {
            let _ = client
                .tokens()
                .create(CreateConnectTokenRequest::new(user()))
                .await;
        }
        "validateToken" => {
            let _ = client
                .tokens()
                .validate(
                    connect_token(),
                    ValidateTokenRequest {
                        app_id: AppSlug::new("google_calendar").unwrap(),
                        account_id: None,
                        oauth_app_id: None,
                    },
                )
                .await;
        }
        "listUsages" => {
            let _ = client
                .usage()
                .list(ListUsageRequest {
                    start_ts: 1000,
                    end_ts: 2000,
                })
                .await;
        }
        "listUsers" => {
            let _ = client.users().list(Default::default()).await;
        }
        "createOauthToken" => return false,
        _ => return false,
    }
    true
}
