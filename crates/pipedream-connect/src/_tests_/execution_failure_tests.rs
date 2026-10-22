//! Best-effort execution error decoding and lossless failure mapping.

use http::Method;
use serde_json::json;

use crate::error::Error;
use crate::request::{ApiPath, ApiRequest, ResponseMode};

use super::support::{Harness, response, static_config};

#[tokio::test]
async fn malformed_execution_fields_remain_execution_failures_and_stay_lossless() {
    let cases = [
        (json!({"error":{"name":"Error","message":42}}), "message"),
        (json!({"error":{"attribution":{}}}), "origin"),
        (
            json!({"error":{"attribution":{"origin":"upstream_api","network_calls_count":1.5}}}),
            "network_calls_count",
        ),
        (
            json!({"error":{"attribution":{"origin":"upstream_api","last_call":{"status":65536}}}}),
            "status",
        ),
    ];
    for (body, failed_field) in cases {
        let harness = Harness::script(static_config(), vec![Ok(response(200, body))]);
        let request = ApiRequest::new(Method::POST, ApiPath::project(&["actions", "run"]))
            .with_response_mode(ResponseMode::Execution { nonblocking: false });
        let error = harness
            .client
            .dispatcher()
            .dispatch(request)
            .await
            .unwrap_err();
        let Error::ExecutionFailed { error, .. } = error else {
            panic!("malformed execution envelope was not classified as a failure")
        };
        let encoded = serde_json::to_value(&*error).unwrap();
        match failed_field {
            "message" => assert_eq!(encoded[failed_field], 42),
            "origin" => assert_eq!(encoded["attribution"][failed_field], "unknown"),
            "network_calls_count" => {
                assert_eq!(encoded["attribution"][failed_field], 1.5)
            }
            "status" => assert_eq!(encoded["attribution"]["last_call"][failed_field], 65536),
            _ => unreachable!(),
        }
        harness.assert_requests(1);
    }
}
