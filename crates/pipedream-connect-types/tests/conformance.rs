//! Schema-generated response and request conformance checks.

use std::fs;

use pipedream_connect_types::*;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

#[path = "_support/conformance_extra.rs"]
mod conformance_extra;
#[path = "_support/query_schemas.rs"]
mod query_schemas;
#[path = "_support/request_schemas.rs"]
mod request_schemas;
#[path = "_support/samples.rs"]
mod samples;

use conformance_extra::HasExtra;
use samples::{Mode, assert_preserved};

const FIXTURE: &str = "tests/fixtures/pipedream_openapi_swagger.json";
type Decode = fn(&str, Value);

fn covered() -> Vec<(&'static str, Decode)> {
    vec![
        ("Account", decode::<Account>),
        ("App", decode::<App>),
        ("AppOverride", decode::<AppOverride>),
        ("ConfigurePropResponse", decode::<ConfigurePropResponse>),
        ("ConnectUsageResponse", decode::<ConnectUsageResponse>),
        ("CreateOAuthTokenResponse", decode::<OAuthTokenResponse>),
        ("CreateTokenResponse", decode::<ConnectTokenResponse>),
        ("DeployTriggerResponse", decode::<DeployTriggerResponse>),
        ("ErrorResponse", decode::<ErrorResponse>),
        ("GetAppCategoryResponse", decode::<AppCategory>),
        ("GetAppResponse", decode::<Data<App>>),
        ("GetAccountsResponse", decode::<GetAccountsResponse>),
        ("GetComponentResponse", decode::<Data<Component>>),
        ("GetComponentsResponse", decode::<Page<Component>>),
        (
            "GetTriggerEventsResponse",
            decode::<GetTriggerEventsResponse>,
        ),
        ("GetTriggerResponse", decode::<GetTriggerResponse>),
        (
            "GetTriggerWebhooksResponse",
            decode::<GetTriggerWebhooksResponse>,
        ),
        (
            "GetTriggerWorkflowsResponse",
            decode::<GetTriggerWorkflowsResponse>,
        ),
        ("GetTriggersResponse", decode::<GetTriggersResponse>),
        ("GetUsersResponse", decode::<GetUsersResponse>),
        ("GetWebhookResponse", decode::<GetWebhookResponse>),
        (
            "GetWebhookWithSigningKeyResponse",
            decode::<GetWebhookWithSigningKeyResponse>,
        ),
        ("ListAccountsResponse", decode::<ListAccountsResponse>),
        (
            "ListAppCategoriesResponse",
            decode::<ListAppCategoriesResponse>,
        ),
        (
            "ListAppOverridesResponse",
            decode::<ListAppOverridesResponse>,
        ),
        ("ListAppsResponse", decode::<ListAppsResponse>),
        ("ListOauthAppsResponse", decode::<ListOauthAppsResponse>),
        ("ListProjectsResponse", decode::<ListProjectsResponse>),
        ("OauthApp", decode::<OauthApp>),
        ("Project", decode::<Project>),
        ("ProjectInfoResponse", decode::<ProjectInfo>),
        ("ReloadPropsResponse", decode::<ReloadPropsResponse>),
        ("RunActionResponse", decode::<RunActionResponse>),
        ("SetWebhookResponse", decode::<SetWebhookResponse>),
        ("ValidateTokenResponse", decode::<ValidateTokenResponse>),
    ]
}

#[test]
fn every_response_union_map_null_and_optional_shape_decodes() {
    let document: Value = serde_json::from_str(&fs::read_to_string(FIXTURE).unwrap()).unwrap();
    let schemas = document["components"]["schemas"].as_object().unwrap();
    for (name, decode) in covered() {
        let schema = &schemas[name];
        for mode in [Mode::Full, Mode::Minimal, Mode::Nullable] {
            for sample in samples::samples(schema, schemas, mode, true) {
                decode(name, sample);
            }
        }
    }
}

#[test]
fn documented_response_examples_decode_directly() {
    let document: Value = serde_json::from_str(&fs::read_to_string(FIXTURE).unwrap()).unwrap();
    for (path, operations) in document["paths"].as_object().unwrap() {
        for (method, operation) in operations.as_object().unwrap() {
            let Some(responses) = operation["responses"].as_object() else {
                continue;
            };
            for (status, response) in responses {
                let Some(content) = response["content"].as_object() else {
                    continue;
                };
                for body in content.values() {
                    for value in response_examples(body) {
                        let name = body["schema"]["$ref"]
                            .as_str()
                            .and_then(|reference| reference.rsplit('/').next());
                        if let Some((name, decode)) = covered()
                            .into_iter()
                            .find(|(covered, _)| Some(*covered) == name)
                        {
                            decode(name, value);
                        } else {
                            let decoded: Value = serde_json::from_value(value.clone())
                                .unwrap_or_else(|error| {
                                    panic!("{path} {method} {status}: {error}")
                                });
                            assert_eq!(decoded, value);
                        }
                    }
                }
            }
        }
    }
}

fn decode<T: DeserializeOwned + Serialize + HasExtra>(name: &str, sample: Value) {
    let decoded: T = serde_json::from_value(sample.clone())
        .unwrap_or_else(|error| panic!("{name}: {error}\n{sample}"));
    decoded.check_extra();
    let encoded = serde_json::to_value(decoded).unwrap();
    assert_preserved(&sample, &encoded, name);
}

fn response_examples(body: &Value) -> Vec<Value> {
    let mut values = Vec::new();
    if let Some(example) = body.get("example") {
        values.push(example.clone());
    }
    if let Some(examples) = body["examples"].as_object() {
        values.extend(
            examples
                .values()
                .filter_map(|example| example.get("value").cloned()),
        );
    }
    values
}
