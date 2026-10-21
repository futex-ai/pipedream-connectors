//! Unit tests for identifier, secret, prop, and pagination contracts.

use std::fmt::Debug;

use serde_json::json;

use crate::{
    AccountId, AppAuthType, AppCategoryId, AppSlug, ComponentKey, ConfigurableProp, ConnectToken,
    CreateConnectTokenRequest, Cursor, DeployedTriggerId, Error, ExternalUserId, OauthAppId,
    PageInfo, PageRequest, ProjectId, PropKind, PropName, SecretJson, SecretString, WebhookId,
};

#[test]
fn identifiers_use_one_validation_path() {
    assert!(ExternalUserId::new("customer/1").is_err());
    assert!(AppCategoryId::new("appcat_demo").is_ok());
    let invalid = serde_json::from_str::<AppCategoryId>("\"appcat/nope\"");
    assert!(invalid.is_err());
    let token = ConnectToken::new("ctok_0123456789abcdef0123456789abcdef");
    assert!(token.is_ok());
    assert!(ConnectToken::new("ctok_ABC").is_err());
    assert!(ProjectId::new("proj_demo").is_ok());
    assert!(AccountId::new("apn_demo").is_ok());
    assert!(DeployedTriggerId::new("dc_demo").is_ok());
    assert!(OauthAppId::new("oa_demo").is_ok());
    assert!(AppSlug::new("google_calendar").is_ok());
    assert!(ComponentKey::new("google_calendar.list").is_ok());
    assert!(PropName::new("account_id").is_ok());
    assert!(WebhookId::new("wh_demo").is_ok());
    assert!(Cursor::new("opaque-cursor").is_ok());
    assert!(ProjectId::new("proj_demo/child").is_err());
}

#[test]
fn unknown_enums_and_props_remain_lossless() {
    let auth: AppAuthType = serde_json::from_str("\"future_auth\"").unwrap();
    assert_eq!(auth, AppAuthType::Other("future_auth".to_owned()));
    let value = json!({
        "name": "new_prop",
        "type": "future_type",
        "future": {"enabled": true}
    });
    let prop: ConfigurableProp = serde_json::from_value(value.clone()).unwrap();
    assert!(matches!(prop.kind, PropKind::Other { .. }));
    assert_eq!(serde_json::to_value(prop).unwrap(), value);
    let app = json!({
        "name_slug": "github",
        "name": "GitHub",
        "id": null,
        "auth_type": null,
        "description": null,
        "img_src": "https://example.com/github.png",
        "custom_fields_json": null,
        "categories": [],
        "featured_weight": 1.0,
        "scope_profiles": [],
        "future_field": {"kept": true}
    });
    let decoded: crate::App = serde_json::from_value(app.clone()).unwrap();
    assert_eq!(decoded.extra.get("future_field"), app.get("future_field"));
    assert_eq!(serde_json::to_value(decoded).unwrap(), app);
}

#[test]
fn secrets_redact_debug_but_serialize_for_wire() {
    let text = SecretString::new("client-secret");
    let debug = format!("{:?}", text);
    assert_eq!(debug, "[REDACTED]");
    assert!(!debug.contains("client-secret"));
    assert_eq!(text.expose_secret(), "client-secret");
    let json_value = SecretJson::new(json!({"token": "value"}));
    assert_eq!(format!("{:?}", json_value), "[REDACTED]");
    assert_eq!(serde_json::to_value(text).unwrap(), json!("client-secret"));
}

#[test]
fn pagination_bounds_and_stuck_cursor_are_typed() {
    assert!(matches!(
        PageRequest::with_limit(0),
        Err(Error::OutOfRange { field: "limit", .. })
    ));
    let request = PageRequest::with_limit(10).unwrap();
    let first = request
        .next(&PageInfo {
            count: 1,
            end_cursor: Some(crate::Cursor::new("next").unwrap()),
            ..PageInfo::default()
        })
        .unwrap()
        .unwrap();
    assert_eq!(first.after.unwrap().as_str(), "next");
    let stuck = PageRequest {
        after: Some(crate::Cursor::new("same").unwrap()),
        ..request
    }
    .next(&PageInfo {
        count: 1,
        end_cursor: Some(crate::Cursor::new("same").unwrap()),
        ..PageInfo::default()
    });
    assert!(matches!(stuck, Err(Error::StuckCursor)));
}

#[test]
fn token_lifetime_and_timestamp_contracts_are_enforced() {
    let user = ExternalUserId::new("user_1").unwrap();
    let request = CreateConnectTokenRequest::new(user);
    assert!(request.clone().with_expires_in(14_400).is_ok());
    assert!(matches!(
        request.clone().with_expires_in(14_401),
        Err(Error::OutOfRange {
            field: "expires_in",
            ..
        })
    ));
    assert!(
        request
            .clone()
            .with_success_redirect_uri("https://example.com/success")
            .is_ok()
    );
    assert!(
        request
            .with_success_redirect_uri("http://example.com/success")
            .is_err()
    );
    let timestamp: crate::Timestamp = serde_json::from_str("1700000000000").unwrap();
    assert_eq!(
        timestamp.as_datetime().timestamp_millis(),
        1_700_000_000_000
    );
    let iso: crate::Timestamp = serde_json::from_str("\"2026-01-01T00:00:00Z\"").unwrap();
    assert_eq!(iso.as_datetime().timestamp(), 1_767_225_600);
}

fn _assert_debug<T: Debug>() {}
