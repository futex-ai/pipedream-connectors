//! Credential-gated development-project smoke tests with explicit opt-in.

#![cfg(feature = "reqwest")]

use std::env;

use pipedream_connect::PipedreamClient;
use pipedream_connect::config::ClientCredentials;
use pipedream_connect_types::{
    AccountId, AppSlug, ComponentKey, ComponentType, ConfiguredProps, DeleteTriggerRequest,
    DeployTriggerRequest, DeployTriggerResponse, Emitter, Environment, ExternalUserId,
    ListDeployedTriggersRequest, ListRegistryRequest, PageRequest, ProjectId,
};

#[tokio::test]
async fn live_pipedream_google_calendar_registry() {
    if env::var("PIPEDREAM_LIVE_TESTS").as_deref() != Ok("1") {
        eprintln!("skipping live Pipedream registry smoke: set PIPEDREAM_LIVE_TESTS=1");
        return;
    }
    let names = [
        "PIPEDREAM_CLIENT_ID",
        "PIPEDREAM_CLIENT_SECRET",
        "PIPEDREAM_PROJECT_ID",
    ];
    let values = names.map(|name| env::var(name).ok().filter(|value| !value.trim().is_empty()));
    let missing: Vec<_> = names
        .iter()
        .zip(&values)
        .filter_map(|(name, value)| value.is_none().then_some(*name))
        .collect();
    if !missing.is_empty() {
        eprintln!(
            "skipping live Pipedream registry smoke: missing {}",
            missing.join(", ")
        );
        return;
    }
    let [Some(client_id), Some(client_secret), Some(project_id)] = values else {
        unreachable!("checked all required credentials")
    };
    let client = PipedreamClient::builder()
        .project_id(ProjectId::new(project_id).expect("valid live project ID"))
        .environment(Environment::Development)
        .client_credentials(
            ClientCredentials::new(client_id, client_secret).expect("valid live credentials"),
        )
        .build()
        .expect("live client configuration");
    let slug = AppSlug::new("google_calendar").unwrap();
    let app = client
        .apps()
        .get(slug.clone())
        .await
        .expect("read the Google Calendar app");
    assert_eq!(app.data.name_slug, slug);
    let request = ListRegistryRequest {
        app: Some(slug),
        ..ListRegistryRequest::default()
    };
    let actions = client
        .actions()
        .list(request.clone())
        .await
        .expect("list Google Calendar actions");
    let triggers = client
        .triggers()
        .list(request)
        .await
        .expect("list Google Calendar triggers");
    assert!(
        !actions.data.is_empty(),
        "Google Calendar has registry actions"
    );
    assert!(
        !triggers.data.is_empty(),
        "Google Calendar has registry triggers"
    );
    assert!(actions.data.iter().all(|component| {
        !component.key.as_str().is_empty()
            && component
                .component_type
                .as_ref()
                .is_some_and(|kind| kind == &ComponentType::Action)
    }));
    assert!(triggers.data.iter().all(|component| {
        !component.key.as_str().is_empty()
            && component
                .component_type
                .as_ref()
                .is_some_and(|kind| kind == &ComponentType::Trigger)
    }));
    eprintln!(
        "read Google Calendar and listed {} actions and {} triggers",
        actions.data.len(),
        triggers.data.len()
    );
}

#[tokio::test]
async fn live_pipedream_google_calendar_trigger_lifecycle() {
    if env::var("PIPEDREAM_LIVE_TESTS").as_deref() != Ok("1") {
        eprintln!("skipping live Pipedream trigger smoke: set PIPEDREAM_LIVE_TESTS=1");
        return;
    }
    let names = [
        "PIPEDREAM_CLIENT_ID",
        "PIPEDREAM_CLIENT_SECRET",
        "PIPEDREAM_PROJECT_ID",
        "PIPEDREAM_EXTERNAL_USER_ID",
        "PIPEDREAM_CONNECTED_ACCOUNT_ID",
    ];
    let values = names.map(|name| env::var(name).ok().filter(|value| !value.trim().is_empty()));
    let missing: Vec<_> = names
        .iter()
        .zip(&values)
        .filter_map(|(name, value)| value.is_none().then_some(*name))
        .collect();
    if !missing.is_empty() {
        eprintln!(
            "skipping live Pipedream trigger smoke: missing {}",
            missing.join(", ")
        );
        return;
    }
    let [
        Some(client_id),
        Some(client_secret),
        Some(project_id),
        Some(external_user_id),
        Some(account_id),
    ] = values
    else {
        unreachable!("checked all required trigger smoke variables")
    };
    let client = PipedreamClient::builder()
        .project_id(ProjectId::new(project_id).expect("valid live project ID"))
        .environment(Environment::Development)
        .client_credentials(
            ClientCredentials::new(client_id, client_secret).expect("valid live credentials"),
        )
        .build()
        .expect("live client configuration");
    let external_user_id = ExternalUserId::new(external_user_id).expect("valid external user ID");
    let account_id = AccountId::new(account_id).expect("valid connected account ID");
    let configured_props: ConfiguredProps = serde_json::from_value(serde_json::json!({
        "googleCalendar": {"authProvisionId": account_id.as_str()}
    }))
    .expect("valid Google Calendar configured props");
    let deployed = client
        .triggers()
        .deploy(DeployTriggerRequest {
            id: ComponentKey::new("google_calendar-new-or-updated-event-instant")
                .expect("valid Google Calendar trigger key"),
            version: None,
            external_user_id: external_user_id.clone(),
            configured_props: Some(configured_props),
            dynamic_props_id: None,
            workflow_id: None,
            webhook_url: Some("http://127.0.0.1:50151/pipedream-trigger-smoke".into()),
            emit_on_deploy: Some(false),
        })
        .await;
    let trigger_id = match deployed {
        Ok(DeployTriggerResponse {
            data: Emitter::DeployedComponent(trigger),
            ..
        }) => trigger.id,
        Ok(other) => {
            cleanup_created_triggers(&client, &external_user_id).await;
            panic!("expected deployed component emitter, got {other:?}");
        }
        Err(error) => {
            cleanup_created_triggers(&client, &external_user_id).await;
            panic!("deploy Google Calendar trigger: {error:?}");
        }
    };
    let lifecycle = async {
        let listed = match client
            .deployed_triggers()
            .list(ListDeployedTriggersRequest {
                external_user_id: external_user_id.clone(),
                page: PageRequest::default(),
                emitter_type: None,
            })
            .await
        {
            Ok(listed) => listed,
            Err(error) => return Err(format!("{error:?}")),
        };
        if !listed.data.iter().any(|emitter| {
            matches!(emitter, Emitter::DeployedComponent(trigger) if trigger.id == trigger_id)
        }) {
            return Err("deployed trigger was not present in the list".to_owned());
        }
        Ok::<(), String>(())
    }
    .await;
    let cleanup = client
        .deployed_triggers()
        .delete(
            trigger_id,
            DeleteTriggerRequest {
                external_user_id: external_user_id.clone(),
                ignore_hook_errors: Some(true),
                missing_ok: true,
            },
        )
        .await;
    if let Err(error) = cleanup {
        panic!("delete Google Calendar trigger: {error:?}");
    }
    if let Err(error) = lifecycle {
        panic!("list Google Calendar trigger: {error}");
    }
}

async fn cleanup_created_triggers(client: &PipedreamClient, external_user_id: &ExternalUserId) {
    let Ok(listed) = client
        .deployed_triggers()
        .list(ListDeployedTriggersRequest {
            external_user_id: external_user_id.clone(),
            page: PageRequest::default(),
            emitter_type: None,
        })
        .await
    else {
        return;
    };
    for emitter in listed.data {
        let Emitter::DeployedComponent(trigger) = emitter else {
            continue;
        };
        let matches_test = trigger
            .component_key
            .as_ref()
            .is_some_and(|key| key.as_str() == "google_calendar-new-or-updated-event-instant")
            && trigger.webhook_urls.as_ref().is_some_and(|urls| {
                urls.iter()
                    .any(|url| url.contains("pipedream-trigger-smoke"))
            });
        if matches_test {
            let _ = client
                .deployed_triggers()
                .delete(
                    trigger.id,
                    DeleteTriggerRequest {
                        external_user_id: external_user_id.clone(),
                        ignore_hook_errors: Some(true),
                        missing_ok: true,
                    },
                )
                .await;
        }
    }
}
