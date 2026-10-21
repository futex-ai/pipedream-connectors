//! Read-only development-project smoke test that skips without all credentials.

#![cfg(feature = "reqwest")]

use std::env;

use pipedream_connect::PipedreamClient;
use pipedream_connect::config::ClientCredentials;
use pipedream_connect_types::{
    AppSlug, ComponentType, Environment, ListRegistryRequest, ProjectId,
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
