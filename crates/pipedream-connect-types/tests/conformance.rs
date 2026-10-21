//! Schema-generated conformance samples for response models.

use std::fs;

use pipedream_connect_types::{
    App, AppCategory, AppOverride, Component, ConnectTokenResponse, Data, ErrorResponse,
    GetTriggerEventsResponse, GetTriggerResponse, GetTriggerWebhooksResponse,
    GetTriggerWorkflowsResponse, GetTriggersResponse, ListAccountsResponse,
    ListAppCategoriesResponse, ListAppOverridesResponse, ListAppsResponse, ListOauthAppsResponse,
    ListProjectsResponse, OAuthTokenResponse, OauthApp, PageEnvelope, Project, ProjectInfo,
    ReloadPropsResponse, RunActionResponse, SetWebhookResponse, UnknownFields,
    ValidateTokenResponse,
};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

const FIXTURE: &str = "tests/fixtures/pipedream_openapi_swagger.json";

#[test]
fn response_schema_samples_decode_without_unknown_documented_fields() {
    let document: Value = serde_json::from_str(&fs::read_to_string(FIXTURE).unwrap()).unwrap();
    let schemas = document["components"]["schemas"].as_object().unwrap();
    let covered = [
        (
            "Account",
            decode::<pipedream_connect_types::Account> as Decode,
        ),
        ("App", decode::<App>),
        ("AppOverride", decode::<AppOverride>),
        (
            "ConfigurePropResponse",
            decode::<pipedream_connect_types::ConfigurePropResponse>,
        ),
        (
            "ConnectUsageResponse",
            decode::<pipedream_connect_types::ConnectUsageResponse>,
        ),
        ("CreateOAuthTokenResponse", decode::<OAuthTokenResponse>),
        ("CreateTokenResponse", decode::<ConnectTokenResponse>),
        (
            "DeployTriggerResponse",
            decode::<pipedream_connect_types::DeployTriggerResponse>,
        ),
        ("ErrorResponse", decode::<ErrorResponse>),
        ("GetAppCategoryResponse", decode::<AppCategory>),
        ("GetAppResponse", decode::<Data<App>>),
        ("GetComponentResponse", decode::<Data<Component>>),
        ("GetComponentsResponse", decode::<PageEnvelope<Component>>),
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
        (
            "GetUsersResponse",
            decode::<pipedream_connect_types::GetUsersResponse>,
        ),
        (
            "GetWebhookResponse",
            decode::<pipedream_connect_types::GetWebhookResponse>,
        ),
        (
            "GetWebhookWithSigningKeyResponse",
            decode::<pipedream_connect_types::GetWebhookWithSigningKeyResponse>,
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
    ];
    for &(name, decoder) in &covered {
        let schema = schemas
            .get(name)
            .unwrap_or_else(|| panic!("missing schema {name}"));
        let mut sample = sample_schema(schema, schemas);
        if name == "GetAppCategoryResponse" {
            sample["id"] = Value::String("appcat_sample".to_owned());
        }
        if name == "ListAppCategoriesResponse" {
            sample[0]["id"] = Value::String("appcat_sample".to_owned());
        }
        if name == "ValidateTokenResponse" {
            sample["project_id"] = Value::String("proj_sample".to_owned());
            sample["oauth_app_id"] = Value::String("oa_sample".to_owned());
        }
        if name == "ProjectInfoResponse" {
            sample["apps"][0]["id"] = Value::String("oa_sample".to_owned());
        }
        decoder(name, sample);
    }
    for (path, operations) in document["paths"].as_object().unwrap() {
        let Some(operations) = operations.as_object() else {
            continue;
        };
        for (method, operation) in operations {
            let Some(responses) = operation["responses"].as_object() else {
                continue;
            };
            for (status, response) in responses {
                let Some(content) = response["content"].as_object() else {
                    continue;
                };
                for (media, body) in content {
                    for value in response_examples(body) {
                        let Some(schema) = body.get("schema") else {
                            continue;
                        };
                        let Some(reference) = schema.get("$ref").and_then(Value::as_str) else {
                            continue;
                        };
                        let name = reference.rsplit('/').next().unwrap();
                        if let Some((_, decoder)) =
                            covered.iter().find(|(covered, _)| *covered == name)
                        {
                            decoder(name, value);
                        }
                        let _ = (path, method, status, media);
                    }
                }
            }
        }
    }
}

trait HasExtra {
    fn extra(&self) -> Option<&UnknownFields>;
}

macro_rules! extra_impl {
    ($($type:ty),+ $(,)?) => {$ (
        impl HasExtra for $type {
            fn extra(&self) -> Option<&UnknownFields> { Some(&self.extra) }
        }
    )+ };
}

extra_impl!(
    pipedream_connect_types::Account,
    App,
    AppOverride,
    pipedream_connect_types::ConfigurePropResponse,
    pipedream_connect_types::ConnectUsageResponse,
    OAuthTokenResponse,
    ConnectTokenResponse,
    ErrorResponse,
    AppCategory,
    OauthApp,
    Project,
    ProjectInfo,
    ReloadPropsResponse,
    RunActionResponse,
    pipedream_connect_types::TriggerWebhooks,
    pipedream_connect_types::TriggerWorkflows,
    ValidateTokenResponse,
);

impl<T> HasExtra for Data<T> {
    fn extra(&self) -> Option<&UnknownFields> {
        Some(&self.extra)
    }
}

impl<T> HasExtra for PageEnvelope<T> {
    fn extra(&self) -> Option<&UnknownFields> {
        Some(&self.extra)
    }
}

impl HasExtra for Vec<AppCategory> {
    fn extra(&self) -> Option<&UnknownFields> {
        None
    }
}

type Decode = fn(&str, Value);

fn decode<T: DeserializeOwned + serde::Serialize + HasExtra>(name: &str, sample: Value) {
    let decoded: T = serde_json::from_value(sample.clone())
        .unwrap_or_else(|error| panic!("{name} sample did not decode: {error}\n{sample}"));
    if let Some(extra) = decoded.extra() {
        assert!(
            extra.is_empty(),
            "{name} placed documented fields in extra: {extra:?}"
        );
    }
    let encoded = serde_json::to_value(decoded).unwrap();
    assert_eq!(sample, encoded, "{name} did not round-trip losslessly");
}

fn response_examples(body: &Value) -> Vec<Value> {
    let mut values = Vec::new();
    if let Some(example) = body.get("example") {
        values.push(example.clone());
    }
    if let Some(examples) = body.get("examples").and_then(Value::as_object) {
        values.extend(
            examples
                .values()
                .filter_map(|example| example.get("value").cloned()),
        );
    }
    values
}

fn sample_schema(schema: &Value, schemas: &serde_json::Map<String, Value>) -> Value {
    if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
        return sample_schema(&schemas[reference.rsplit('/').next().unwrap()], schemas);
    }
    if let Some(example) = schema.get("example") {
        return example.clone();
    }
    if let Some(one_of) = schema.get("oneOf").and_then(Value::as_array) {
        return sample_schema(&one_of[0], schemas);
    }
    if let Some(any_of) = schema.get("anyOf").and_then(Value::as_array) {
        return sample_schema(&any_of[0], schemas);
    }
    if let Some(all_of) = schema.get("allOf").and_then(Value::as_array) {
        let mut object = Map::new();
        for part in all_of {
            if let Value::Object(fields) = sample_schema(part, schemas) {
                object.extend(fields);
            }
        }
        return Value::Object(object);
    }
    if let Some(values) = schema.get("enum").and_then(Value::as_array) {
        return values[0].clone();
    }
    match schema.get("type").and_then(Value::as_str) {
        Some("object") | None => {
            let mut object = Map::new();
            if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
                for (name, property) in properties {
                    object.insert(name.clone(), sample_schema(property, schemas));
                }
            }
            Value::Object(object)
        }
        Some("array") => Value::Array(vec![sample_schema(&schema["items"], schemas)]),
        Some("integer") => Value::from(1),
        Some("number") => Value::from(1.0),
        Some("boolean") => Value::from(false),
        Some("string") => Value::String(sample_string(schema)),
        _ => Value::Null,
    }
}

fn sample_string(schema: &Value) -> String {
    if let Some(pattern) = schema.get("pattern").and_then(Value::as_str) {
        for (needle, value) in [
            ("proj_", "proj_sample"),
            ("apn_", "apn_sample"),
            ("dc_", "dc_sample"),
            ("oa_", "oa_sample"),
            ("appcat_", "appcat_sample"),
            ("ctok_", "ctok_0123456789abcdef0123456789abcdef"),
            ("p_", "p_sample"),
        ] {
            if pattern.contains(needle) {
                return value.to_owned();
            }
        }
    }
    match schema.get("format").and_then(Value::as_str) {
        Some("date-time") => "2026-01-01T00:00:00Z".to_owned(),
        Some("uri") | Some("uri-reference") => "https://example.com/hook".to_owned(),
        Some("email") => "user@example.com".to_owned(),
        _ => "sample".to_owned(),
    }
}
