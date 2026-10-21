//! Schema-generated request bodies for every covered JSON operation.

use std::fs;

use pipedream_connect_types::*;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use super::samples::{self, Mode, assert_preserved};

type Encode = fn(Value) -> Value;

fn encode<T: DeserializeOwned + Serialize>(sample: Value) -> Value {
    let model: T = serde_json::from_value(sample.clone())
        .unwrap_or_else(|error| panic!("request {sample}: {error}"));
    serde_json::to_value(model).unwrap()
}

#[test]
fn schema_request_samples_encode_the_same_documented_body() {
    let document: Value =
        serde_json::from_str(&fs::read_to_string(super::FIXTURE).unwrap()).unwrap();
    let schemas = document["components"]["schemas"].as_object().unwrap();
    let covered: Vec<(&str, Encode)> = vec![
        ("CreateAccountOpts", encode::<CreateAccountRequest>),
        ("CreateAppOverrideOpts", encode::<CreateAppOverrideRequest>),
        ("CreateOAuthTokenOpts", encode::<CreateOauthTokenRequest>),
        ("CreateOauthAppOpts", encode::<CreateOauthAppRequest>),
        ("CreateProjectOpts", encode::<CreateProjectRequest>),
        ("CreateTokenOpts", encode::<CreateConnectTokenRequest>),
        ("ConfigurePropOpts", encode::<ConfigurePropRequest>),
        ("DeployTriggerOpts", encode::<DeployTriggerRequest>),
        ("ReloadPropsOpts", encode::<ReloadPropsRequest>),
        ("RunActionOpts", encode::<RunActionRequest>),
        ("SetWebhookOpts", encode::<SetWebhookRequest>),
        ("UpdateAppOverrideOpts", encode::<UpdateAppOverrideRequest>),
        ("UpdateOauthAppOpts", encode::<UpdateOauthAppRequest>),
        ("UpdateProjectLogoOpts", encode::<UpdateProjectLogoRequest>),
        ("UpdateProjectOpts", encode::<UpdateProjectRequest>),
        ("UpdateTriggerOpts", encode::<UpdateTriggerRequest>),
        (
            "UpdateTriggerWebhooksOpts",
            encode::<UpdateTriggerWebhooksRequest>,
        ),
        (
            "UpdateTriggerWorkflowsOpts",
            encode::<UpdateTriggerWorkflowsRequest>,
        ),
    ];
    let mut operations = 0;
    for methods in document["paths"].as_object().unwrap().values() {
        for operation in methods.as_object().unwrap().values() {
            let schema = &operation["requestBody"]["content"]["application/json"]["schema"];
            let Some(reference) = schema["$ref"].as_str() else {
                continue;
            };
            let name = reference.rsplit('/').next().unwrap();
            let (_, encoder) = covered
                .iter()
                .find(|(covered, _)| *covered == name)
                .unwrap_or_else(|| panic!("uncovered request schema {name}"));
            operations += 1;
            for mode in [Mode::Full, Mode::Minimal, Mode::Nullable] {
                for sample in samples::samples(schema, schemas, mode, false) {
                    let encoded = encoder(sample.clone());
                    let mut expected = sample;
                    expected.as_object_mut().unwrap().retain(|key, value| {
                        !value.is_null()
                            || (name == "UpdateAppOverrideOpts" && key == "oauth_app_id")
                    });
                    if name == "CreateTokenOpts" && expected.get("expires_in").is_none() {
                        assert_eq!(encoded["expires_in"], Value::from(14_400));
                        expected["expires_in"] = Value::from(14_400);
                    }
                    assert_preserved(&expected, &encoded, name);
                }
            }
        }
    }
    assert_eq!(operations, 22);
}
