//! Every status mapping, structured execution errors, and header parsing.

use std::time::{Duration, UNIX_EPOCH};

use http::{HeaderValue, Method};
use pipedream_connect_types::ErrorOrigin;
use serde_json::json;

use crate::error::Error;
use crate::request::{ApiPath, ApiRequest, ResponseMode};

use super::support::{Harness, response, static_config};

#[tokio::test]
async fn known_statuses_keep_typed_categories_and_redacted_messages() {
    for status in [
        300, 301, 302, 304, 307, 308, 400, 401, 403, 404, 408, 409, 422, 429, 500, 502, 503, 599,
    ] {
        let harness = Harness::script(
            static_config(),
            vec![Ok(response(
                status,
                json!({"error":"provider-secret", "code":"provider-code", "details":{"token":"body-secret"}}),
            ))],
        );
        let error = harness.get().await.unwrap_err();
        assert!(!format!("{error:?} {error}").contains("secret"));
        match (status, error) {
            (401, Error::Unauthorized { message })
            | (403, Error::Forbidden { message })
            | (404, Error::NotFound { message }) => {
                assert_eq!(message.as_deref(), Some("provider-secret"))
            }
            (408 | 500..=599, Error::Unavailable { status: actual, .. }) => {
                assert_eq!(actual, Some(status))
            }
            (
                429,
                Error::RateLimited {
                    retry_after,
                    limit,
                    remaining,
                    reset,
                    ..
                },
            ) => assert_eq!(
                (retry_after, limit, remaining, reset),
                (None, None, None, None)
            ),
            (
                _,
                Error::Rejected {
                    status: actual,
                    code,
                    message,
                },
            ) => {
                assert_eq!(actual, status);
                assert_eq!(code.as_deref(), Some("provider-code"));
                assert_eq!(message.as_deref(), Some("provider-secret"));
            }
            (_, error) => panic!("wrong category for {status}: {error:?}"),
        }
        harness.assert_requests(1);
    }
}

#[tokio::test]
async fn a_structured_error_without_execution_fields_keeps_http_status_mapping() {
    let harness = Harness::script(
        static_config(),
        vec![Ok(response(400, json!({"error":{"message":"bad"}})))],
    );
    assert!(matches!(
        harness.get().await,
        Err(Error::Rejected { status: 400, .. })
    ));
    harness.assert_requests(1);
}

#[tokio::test]
async fn malformed_and_non_json_errors_still_map_from_the_status() {
    for body in [
        b"upstream secret html".to_vec(),
        b"{".to_vec(),
        b"null".to_vec(),
        b"{\"error\":{\"future\":true}}".to_vec(),
    ] {
        let mut reply = response(403, json!({}));
        reply.body = body;
        let harness = Harness::script(static_config(), vec![Ok(reply)]);
        assert!(matches!(
            harness.get().await,
            Err(Error::Forbidden { message: None })
        ));
        harness.assert_requests(1);
    }
}

#[tokio::test]
async fn retry_after_and_rate_limit_headers_are_optional_and_case_insensitive() {
    for (header, expected) in [
        ("25".to_owned(), Some(25)),
        (
            httpdate::fmt_http_date(UNIX_EPOCH + Duration::from_secs(1042)),
            Some(42),
        ),
        (
            httpdate::fmt_http_date(UNIX_EPOCH + Duration::from_secs(999)),
            Some(0),
        ),
        ("garbage".to_owned(), None),
        ("-5".to_owned(), None),
        ("+5".to_owned(), None),
        ("18446744073709551616".to_owned(), None),
    ] {
        let mut reply = response(429, json!({}));
        reply
            .headers
            .insert("Retry-After", HeaderValue::from_str(&header).unwrap());
        reply
            .headers
            .insert("X-RateLimit-Limit", HeaderValue::from_static("100"));
        reply
            .headers
            .insert("X-RateLimit-Remaining", HeaderValue::from_static("0"));
        reply
            .headers
            .insert("X-RateLimit-Reset", HeaderValue::from_static("1200"));
        let harness = Harness::script(static_config(), vec![Ok(reply)]);
        assert!(
            matches!(harness.get().await, Err(Error::RateLimited { retry_after, limit: Some(100), remaining: Some(0), reset: Some(1200), .. }) if retry_after == expected.map(Duration::from_secs))
        );
        harness.assert_requests(1);
    }
    let mut reply = response(429, json!({}));
    reply
        .headers
        .insert("x-ratelimit-limit", HeaderValue::from_static("-1"));
    reply
        .headers
        .insert("x-ratelimit-remaining", HeaderValue::from_static("+2"));
    reply.headers.insert(
        "x-ratelimit-reset",
        HeaderValue::from_bytes(&[0xff]).unwrap(),
    );
    let harness = Harness::script(static_config(), vec![Ok(reply)]);
    assert!(matches!(
        harness.get().await,
        Err(Error::RateLimited {
            limit: None,
            remaining: None,
            reset: None,
            ..
        })
    ));
    harness.assert_requests(1);
}

#[tokio::test]
async fn execution_attribution_and_provider_text_stay_available_as_data() {
    for (origin, expected) in [
        ("component_code", ErrorOrigin::ComponentCode),
        ("upstream_api", ErrorOrigin::UpstreamApi),
        ("network_io", ErrorOrigin::NetworkIo),
        ("response_parsing", ErrorOrigin::ResponseParsing),
        ("future", ErrorOrigin::Other("future".into())),
    ] {
        for status in [200, 400] {
            let reply = response(
                status,
                json!({"error":{"name":"provider-name", "message":"provider-secret", "attribution":{"origin":origin, "network_calls_count":2, "last_call":{"status":401}}}}),
            );
            let harness = Harness::script(static_config(), vec![Ok(reply)]);
            let request = ApiRequest::new(Method::POST, ApiPath::project(&["actions", "run"]))
                .with_response_mode(ResponseMode::Execution);
            let error = harness
                .client
                .dispatcher()
                .dispatch(request)
                .await
                .unwrap_err();
            assert!(!format!("{error:?} {error}").contains("provider-secret"));
            let Error::ExecutionFailed {
                error: execution,
                status: actual,
            } = error
            else {
                panic!("missing execution error")
            };
            assert_eq!(actual, status);
            assert_eq!(execution.message.as_deref(), Some("provider-secret"));
            let attribution = execution.attribution.unwrap();
            assert_eq!(attribution.origin, expected);
            assert_eq!(attribution.network_calls_count, Some(2));
            assert_eq!(attribution.last_call.unwrap().status, Some(401));
            harness.assert_requests(1);
        }
    }
}

#[tokio::test]
async fn accepted_and_redirect_results_require_the_operation_contract() {
    let harness = Harness::script(
        static_config(),
        vec![
            Ok(response(202, json!({}))),
            Ok(response(202, json!({"error":{"name":"still-pending"}}))),
            Ok(response(202, json!({}))),
            Ok(response(302, json!({}))),
            Ok(response(204, json!(null))),
        ],
    );
    assert_eq!(harness.get().await.unwrap().status, 202);
    let operation = |mode| {
        ApiRequest::new(Method::POST, ApiPath::project(&["actions", "run"]))
            .with_response_mode(mode)
    };
    assert!(matches!(
        harness
            .client
            .dispatcher()
            .dispatch(operation(ResponseMode::Execution))
            .await,
        Err(Error::Accepted)
    ));
    assert!(matches!(
        harness
            .client
            .dispatcher()
            .dispatch(operation(ResponseMode::Execution))
            .await,
        Err(Error::Accepted)
    ));
    assert_eq!(
        harness
            .client
            .dispatcher()
            .dispatch(operation(ResponseMode::Redirect))
            .await
            .unwrap()
            .status,
        302
    );
    assert_eq!(
        harness
            .client
            .dispatcher()
            .dispatch(operation(ResponseMode::Bytes))
            .await
            .unwrap()
            .status,
        204
    );
    harness.assert_requests(5);
}

#[tokio::test]
async fn an_execution_error_object_cannot_be_an_empty_success() {
    let harness = Harness::script(
        static_config(),
        vec![Ok(response(
            200,
            json!({"error":{"name":"Error","message":"provider-secret"}}),
        ))],
    );
    let request = ApiRequest::new(Method::POST, ApiPath::project(&["actions", "run"]))
        .with_response_mode(ResponseMode::Execution);
    let error = harness
        .client
        .dispatcher()
        .dispatch(request)
        .await
        .unwrap_err();
    assert!(matches!(error, Error::ExecutionFailed { .. }));
    harness.assert_requests(1);
}
