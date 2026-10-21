//! Configuration and reload contracts on all three registry resource groups.

use std::num::NonZeroU32;
use std::time::Duration;

use pipedream_connect_types::{
    ConfigurePropRequest, ConfigurePropResponse, ErrorOrigin, ReloadPropsRequest,
    ReloadPropsResponse,
};
use serde_json::json;

use crate::client::PipedreamClient;
use crate::error::{Error, Result};
use crate::resources::test_support::{assert_post, configure, configured, reload, reloaded};
use crate::retry::RetryPolicy;
use crate::tests::support::{Harness, config, response, token};

#[derive(Clone, Copy)]
enum Resource {
    Components,
    Actions,
    Triggers,
}

impl Resource {
    fn all() -> [Self; 3] {
        [Self::Components, Self::Actions, Self::Triggers]
    }

    fn path(self) -> &'static str {
        match self {
            Self::Components => "components",
            Self::Actions => "actions",
            Self::Triggers => "triggers",
        }
    }

    async fn configure(
        self,
        client: &PipedreamClient,
        request: ConfigurePropRequest,
    ) -> Result<ConfigurePropResponse> {
        match self {
            Self::Components => client.components().configure(request).await,
            Self::Actions => client.actions().configure(request).await,
            Self::Triggers => client.triggers().configure(request).await,
        }
    }

    async fn reload(
        self,
        client: &PipedreamClient,
        request: ReloadPropsRequest,
    ) -> Result<ReloadPropsResponse> {
        match self {
            Self::Components => client.components().reload_props(request).await,
            Self::Actions => client.actions().reload_props(request).await,
            Self::Triggers => client.triggers().reload_props(request).await,
        }
    }
}

#[tokio::test]
async fn configure_and_reload_send_exact_bodies_and_keep_lossless_results() {
    for resource in Resource::all() {
        for blocking in [None, Some(true), Some(false)] {
            let harness =
                Harness::new(vec![response(200, configured()), response(200, reloaded())]);
            let (request, expected) = configure(blocking);
            let result = resource.configure(&harness.client, request).await.unwrap();
            assert_post(
                harness.request(1),
                &format!("{}/configure", resource.path()),
                &expected,
            );
            assert_eq!(
                result.errors.as_deref(),
                Some(["provider error".to_owned()].as_slice())
            );
            assert_eq!(result.context, Some(json!({"next":2})));
            let encoded = serde_json::to_value(result).unwrap();
            assert_eq!(encoded["options"], configured()["options"]);
            assert_eq!(encoded["stringOptions"], json!(["primary"]));
            assert_eq!(encoded["observations"][0]["msg"], "provider log");
            assert_eq!(encoded["future_response"], json!({"keep":true}));
            let (request, expected) = reload(blocking);
            let result = resource.reload(&harness.client, request).await.unwrap();
            assert_post(
                harness.request(2),
                &format!("{}/props", resource.path()),
                &expected,
            );
            let encoded = serde_json::to_value(result).unwrap();
            assert_eq!(
                encoded["dynamicProps"]["configurableProps"],
                reloaded()["dynamicProps"]["configurableProps"]
            );
            assert_eq!(encoded["dynamicProps"]["future_dynamic"], 1);
            assert_eq!(encoded["future_response"], json!({"keep":true}));
            harness.assert_requests(3);
        }
    }
}

#[tokio::test]
async fn nonblocking_configure_and_reload_accept_202_without_decoding_a_result() {
    for resource in Resource::all() {
        for blocking in [None, Some(false), Some(true)] {
            let mut reply = response(202, json!({}));
            reply.body = b"accepted without JSON".to_vec();
            let harness = Harness::new(vec![reply.clone(), reply]);
            let (request, expected) = configure(blocking);
            let result = resource.configure(&harness.client, request).await;
            if blocking == Some(true) {
                assert!(matches!(result, Err(Error::Internal(_))));
            } else {
                assert!(matches!(result, Err(Error::Accepted)));
            }
            assert_post(
                harness.request(1),
                &format!("{}/configure", resource.path()),
                &expected,
            );
            let (request, expected) = reload(blocking);
            let result = resource.reload(&harness.client, request).await;
            if blocking == Some(true) {
                assert!(matches!(result, Err(Error::Internal(_))));
            } else {
                assert!(matches!(result, Err(Error::Accepted)));
            }
            assert_post(
                harness.request(2),
                &format!("{}/props", resource.path()),
                &expected,
            );
            harness.assert_requests(3);
        }
    }
}

#[tokio::test]
async fn blocking_configure_and_reload_decode_a_202_body() {
    for resource in Resource::all() {
        let harness = Harness::new(vec![response(202, json!({})), response(202, json!({}))]);
        let (request, _) = configure(Some(true));
        assert!(resource.configure(&harness.client, request).await.is_ok());
        let (request, _) = reload(Some(true));
        assert!(resource.reload(&harness.client, request).await.is_ok());
        harness.assert_requests(3);
    }
}

#[tokio::test]
async fn configure_and_reload_return_structured_execution_failures() {
    for resource in Resource::all() {
        for status in [200, 400, 503] {
            let failure = json!({"error":{"name":"ProviderError","message":"provider-secret",
                "attribution":{"origin":"component_code","network_calls_count":1}}});
            let harness = Harness::new(vec![
                response(status, failure.clone()),
                response(status, failure),
            ]);
            let (configure, _) = configure(Some(true));
            let (reload, _) = reload(Some(true));
            let errors = [
                resource
                    .configure(&harness.client, configure)
                    .await
                    .unwrap_err(),
                resource.reload(&harness.client, reload).await.unwrap_err(),
            ];
            for (error, expected_status) in errors.into_iter().zip([status, status]) {
                assert!(!format!("{error:?} {error}").contains("provider-secret"));
                let Error::ExecutionFailed {
                    error,
                    status: actual,
                } = error
                else {
                    panic!("expected structured execution error")
                };
                assert_eq!(actual, expected_status);
                assert_eq!(error.message.as_deref(), Some("provider-secret"));
                assert_eq!(
                    error.attribution.unwrap().origin,
                    ErrorOrigin::ComponentCode
                );
            }
            harness.assert_requests(3);
        }
    }
}

#[tokio::test]
async fn configure_and_reload_map_http_errors_without_mutation_retries() {
    for resource in Resource::all() {
        for status in [429, 503] {
            let harness = Harness::script(
                config().with_retry_policy(RetryPolicy::new(
                    NonZeroU32::new(3).unwrap(),
                    Duration::from_secs(30),
                    Duration::ZERO,
                )),
                vec![
                    Ok(token("access-secret", 3600)),
                    Ok(response(status, json!({"error":"failed"}))),
                    Ok(response(status, json!({"error":"failed"}))),
                ],
            );
            let (configure, _) = configure(Some(true));
            let (reload, _) = reload(Some(true));
            let errors = [
                resource
                    .configure(&harness.client, configure)
                    .await
                    .unwrap_err(),
                resource.reload(&harness.client, reload).await.unwrap_err(),
            ];
            for error in errors {
                assert!(matches!(
                    (status, error),
                    (429, Error::RateLimited { .. })
                        | (503, Error::Unavailable { status: Some(503) })
                ));
            }
            harness.assert_requests(3);
        }
    }
}
