//! Literal request and response samples shared by resource tests.

use http::Method;
use pipedream_connect_types::{ConfigurePropRequest, ReloadPropsRequest, RunActionRequest};
use serde_json::{Value, json};

use crate::transport::HttpRequest;

pub(super) fn app() -> Value {
    json!({"name_slug":"google_calendar","name":"Google Calendar","img_src":"https://example.com/logo",
        "auth_type":"future_auth","future":{"keep":true}})
}

pub(super) fn component() -> Value {
    json!({"key":"google_calendar-list-events","name":"List Events","version":"0.2.3",
        "component_type":"future_component","stash":"future_stash",
        "configurable_props":[{"name":"futureProp","type":"future_kind",
            "remoteOptions":"new_format","provider":{"keep":[1,true,"data"]}}],
        "future":{"keep":true}})
}

pub(super) fn page(value: Value) -> Value {
    json!({"data":[value],"page_info":{"count":1,"end_cursor":"next","future_count":42},"future_page":{"keep":true}})
}

pub(super) fn configured() -> Value {
    json!({"options":[{"label":"Team","value":"team","future":true},{"__lv":{"label":"Item","value":false},"outer":1}],
        "stringOptions":["primary"],"observations":[{"k":"console.log","ts":1000,"msg":"provider log"}],
        "context":{"next":2},"errors":["provider error"],"future_response":{"keep":true}})
}

pub(super) fn reloaded() -> Value {
    json!({"dynamicProps":{"id":"dp_new","configurableProps":[{"name":"unknown","type":"future_kind","future":{"keep":true}}],"future_dynamic":1},
        "observations":[],"errors":[],"future_response":{"keep":true}})
}

pub(super) fn configure(blocking: Option<bool>) -> (ConfigurePropRequest, Value) {
    let mut body = json!({"id":"google_calendar-list-events","version":"0.2.3","external_user_id":"customer_1",
        "prop_name":"CalendarID","configured_props":{"google_calendar":{"authProvisionId":"apn_Caller","future":true},"calendar":"primary"},
        "dynamic_props_id":"dp_prior","page":1.5,"prev_context":{"cursor":"prior"},"query":"team & schedule"});
    if let Some(blocking) = blocking {
        body["blocking"] = json!(blocking);
    }
    (serde_json::from_value(body.clone()).unwrap(), body)
}

pub(super) fn reload(blocking: Option<bool>) -> (ReloadPropsRequest, Value) {
    let mut body = json!({"id":"google_calendar-list-events","version":"0.2.3","external_user_id":"customer_1",
        "configured_props":{"google_calendar":{"authProvisionId":"apn_Caller"},"calendar":"primary"},"dynamic_props_id":"dp_prior"});
    if let Some(blocking) = blocking {
        body["blocking"] = json!(blocking);
    }
    (serde_json::from_value(body.clone()).unwrap(), body)
}

pub(super) fn run(blocking: Option<bool>, stash: Value) -> (RunActionRequest, Value) {
    let mut body = json!({"id":"google_calendar-list-events","version":"0.2.3","external_user_id":"customer_1",
        "configured_props":{"google_calendar":{"authProvisionId":"apn_Caller"},"calendar":"primary"},"dynamic_props_id":"dp_prior","stash_id":stash});
    if let Some(blocking) = blocking {
        body["blocking"] = json!(blocking);
    }
    (serde_json::from_value(body.clone()).unwrap(), body)
}

pub(super) fn assert_get(request: HttpRequest, url: &str) {
    assert_eq!(request.method, Method::GET);
    assert_eq!(request.url.expose_secret(), url);
    assert!(request.body.is_none());
    assert_eq!(request.headers["x-pd-environment"], "development");
    assert_eq!(request.headers["authorization"], "Bearer access-secret");
}

pub(super) fn assert_post(request: HttpRequest, path: &str, expected: &Value) {
    assert_eq!(request.method, Method::POST);
    assert_eq!(
        request.url.expose_secret(),
        format!("https://api.pipedream.com/v1/connect/proj_Test123/{path}")
    );
    assert_eq!(
        serde_json::from_slice::<Value>(request.body.as_ref().unwrap()).unwrap(),
        *expected
    );
    assert_eq!(request.headers["x-pd-environment"], "development");
    assert_eq!(request.headers["authorization"], "Bearer access-secret");
    assert_eq!(request.headers["content-type"], "application/json");
}
