//! OpenAPI and protocol-table coverage for every Connect operation.

mod openapi_coverage {
    #[path = "support.rs"]
    mod support;
    #[path = "wire.rs"]
    mod wire;

    use std::collections::HashMap;
    use std::path::Path;
    use std::sync::Arc;

    use async_trait::async_trait;
    use http::{HeaderMap, StatusCode};
    use pipedream_connect::PipedreamClient;
    use pipedream_connect::config::{Authentication, ClientConfig, ClientCredentials};
    use pipedream_connect::runtime::{SystemClock, TokioSleeper};
    use pipedream_connect::transport::{HttpRequest, HttpResponse, HttpTransport};
    use pipedream_connect_types::{Environment, ProjectId, SecretString};
    use serde_json::Value;

    use self::support::{Recorder, invoke};
    use self::wire::Wire;

    const OPENAPI: &str =
        include_str!("../../pipedream-connect-types/tests/fixtures/pipedream_openapi_swagger.json");

    #[tokio::test]
    async fn covered_operations_have_documentation_and_golden_wire_rows() {
        let openapi = openapi_operations();
        let documented = documented_operations();
        assert_eq!(
            documented.len(),
            openapi.len(),
            "operation table count differs"
        );
        for (id, (method, path)) in &openapi {
            let documented_row = documented
                .get(id)
                .unwrap_or_else(|| panic!("{id} is missing from protocol tables"));
            assert_eq!(
                documented_row,
                &(method.clone(), path.clone()),
                "wrong table row for {id}"
            );
            assert!(
                Wire::for_operation(id).is_some(),
                "{id} has no golden wire row"
            );
        }
        for id in documented.keys() {
            assert!(
                openapi.contains_key(id),
                "protocol tables name unknown operation {id}"
            );
        }

        let recorder = Arc::new(Recorder::default());
        let client = static_client(recorder.clone());
        for (id, (method, template)) in &openapi {
            if id == "createOauthToken" {
                continue;
            }
            let expected = Wire::for_operation(id).expect("checked above");
            recorder.clear();
            assert!(invoke(&client, id).await, "no client method for {id}");
            let request = recorder.take_one(id);
            expected.assert_request(&request, method, template);
        }

        let expected = Wire::for_operation("createOauthToken").expect("OAuth row");
        recorder.clear();
        let client = credentials_client(recorder.clone());
        let _ = client.projects().info().await;
        let request = recorder.take_oauth();
        expected.assert_request(&request, "POST", "/v1/oauth/token");
    }

    fn static_client(recorder: Arc<Recorder>) -> PipedreamClient {
        let config = ClientConfig::new(
            ProjectId::new("proj_Test123").expect("project ID"),
            Environment::Development,
            Authentication::StaticBearer(SecretString::new("access-token")),
        )
        .expect("static configuration");
        PipedreamClient::new(
            config,
            recorder,
            Arc::new(SystemClock::default()),
            Arc::new(TokioSleeper),
        )
        .expect("client")
    }

    fn credentials_client(recorder: Arc<Recorder>) -> PipedreamClient {
        let config = ClientConfig::new(
            ProjectId::new("proj_Test123").expect("project ID"),
            Environment::Development,
            Authentication::ClientCredentials(
                ClientCredentials::new("client-id", "client-secret").expect("credentials"),
            ),
        )
        .expect("credential configuration");
        PipedreamClient::new(
            config,
            recorder,
            Arc::new(SystemClock::default()),
            Arc::new(TokioSleeper),
        )
        .expect("client")
    }

    fn openapi_operations() -> HashMap<String, (String, String)> {
        let document: Value = serde_json::from_str(OPENAPI).expect("valid OpenAPI fixture");
        let paths = document["paths"].as_object().expect("OpenAPI paths");
        let mut operations = HashMap::new();
        for (path, item) in paths {
            let Some(item) = item.as_object() else {
                continue;
            };
            for (method, operation) in item {
                if !matches!(method.as_str(), "get" | "post" | "put" | "patch" | "delete") {
                    continue;
                }
                let id = operation["operationId"].as_str().expect("operationId");
                if path.starts_with("/v1/connect/") || id == "createOauthToken" {
                    assert!(
                        operations
                            .insert(id.to_owned(), (method.to_uppercase(), path.clone()))
                            .is_none(),
                        "duplicate OpenAPI operation {id}"
                    );
                }
            }
        }
        operations
    }

    fn documented_operations() -> HashMap<String, (String, String)> {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/protocol/resources");
        let mut operations = HashMap::new();
        for entry in std::fs::read_dir(directory).expect("protocol resource directory") {
            let path = entry.expect("protocol directory entry").path();
            if path.extension().and_then(|value| value.to_str()) != Some("md") {
                continue;
            }
            let text = std::fs::read_to_string(path).expect("protocol resource table");
            for line in text.lines() {
                let cells: Vec<_> = line.split('|').map(str::trim).collect();
                if cells.len() < 5
                    || !matches!(cells[1], "GET" | "POST" | "PUT" | "PATCH" | "DELETE")
                {
                    continue;
                }
                let Some(operation_id) = cells[3]
                    .strip_prefix('`')
                    .and_then(|value| value.strip_suffix('`'))
                else {
                    continue;
                };
                assert!(
                    operations
                        .insert(
                            operation_id.to_owned(),
                            (cells[1].to_owned(), cells[2].trim_matches('`').to_owned())
                        )
                        .is_none(),
                    "operation {operation_id} appears more than once in protocol tables"
                );
            }
        }
        operations
    }

    #[async_trait]
    impl HttpTransport for Recorder {
        async fn send(
            &self,
            request: HttpRequest,
        ) -> pipedream_connect::error::Result<HttpResponse> {
            let oauth = request.url.expose_secret().ends_with("/v1/oauth/token");
            self.record(request);
            let (status, body) = if oauth {
                (
                    StatusCode::OK,
                    br#"{"access_token":"bearer","token_type":"Bearer","expires_in":3600}"#
                        .to_vec(),
                )
            } else {
                (StatusCode::OK, b"{}".to_vec())
            };
            Ok(HttpResponse {
                status,
                headers: HeaderMap::new(),
                body,
            })
        }
    }
}
