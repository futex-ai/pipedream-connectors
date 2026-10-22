//! Conformance samples from operation query parameters.

use std::fs;

use pipedream_connect_types::*;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use super::samples::{self, Mode, assert_preserved};

type Encode = fn(Value) -> Value;

fn encode<T: DeserializeOwned + Serialize>(sample: Value) -> Value {
    let value: T = serde_json::from_value(sample).unwrap();
    serde_json::to_value(value).unwrap()
}

#[test]
fn operation_query_samples_match_encoded_models() {
    let document: Value =
        serde_json::from_str(&fs::read_to_string(super::FIXTURE).unwrap()).unwrap();
    let schemas = document["components"]["schemas"].as_object().unwrap();
    let mut count = 0;
    for methods in document["paths"].as_object().unwrap().values() {
        for operation in methods.as_object().unwrap().values() {
            let Some(parameters) = operation["parameters"].as_array() else {
                continue;
            };
            let mut schema = json!({"type":"object","properties":{},"required":[]});
            for parameter in parameters
                .iter()
                .filter(|parameter| parameter["in"] == "query")
            {
                let name = parameter["name"].as_str().unwrap();
                schema["properties"][name] = parameter["schema"].clone();
            }
            if schema["properties"].as_object().unwrap().is_empty() {
                continue;
            }
            let name = operation["operationId"].as_str().unwrap();
            let encoder: Encode = match name {
                "listApps" | "listAppsBase" => encode::<ListAppsRequest>,
                "listAccounts" => encode::<ListAccountsRequest>,
                "retrieveAccount" => encode::<GetAccountRequest>,
                "listUserAccounts" => encode::<ListUserAccountsRequest>,
                "listAppOverrides" => encode::<ListAppOverridesRequest>,
                "listComponents" => encode::<ListComponentsRequest>,
                "listActions" | "listTriggers" => encode::<ListRegistryRequest>,
                "retrieveComponent" | "retrieveAction" | "retrieveTrigger" => {
                    encode::<GetComponentRequest>
                }
                "listDeployedTriggers" => encode::<ListDeployedTriggersRequest>,
                "deleteDeployedTrigger" => encode::<DeleteTriggerRequest>,
                "listDeployedTriggerEvents" => encode::<ListTriggerEventsRequest>,
                "retrieveDeployedTrigger"
                | "updateDeployedTrigger"
                | "listDeployedTriggerWorkflows"
                | "updateDeployedTriggerWorkflows"
                | "listDeployedTriggerWebhooks"
                | "updateDeployedTriggerWebhooks"
                | "retrieveDeployedTriggerWebhook"
                | "regenerateDeployedTriggerWebhookSigningKey" => encode::<DeployedTriggerQuery>,
                "listOauthApps" => encode::<ListOauthAppsRequest>,
                "listProjects" => encode::<ListProjectsRequest>,
                "downloadFileStashFile" => encode::<DownloadFileStashRequest>,
                "validateToken" => encode::<ValidateTokenRequest>,
                "listUsages" => encode::<ListUsageRequest>,
                "listUsers" => encode::<ListUsersRequest>,
                _ => panic!("uncovered query {name}"),
            };
            count += 1;
            for direction in ["after", "before"] {
                let mut direction_schema = schema.clone();
                direction_schema["properties"]
                    .as_object_mut()
                    .unwrap()
                    .remove(direction);
                for sample in samples::samples(&direction_schema, schemas, Mode::Full, false) {
                    let encoded = encoder(sample.clone());
                    assert_preserved(&sample, &encoded, name);
                }
            }
        }
    }
    assert_eq!(count, 29);
}
