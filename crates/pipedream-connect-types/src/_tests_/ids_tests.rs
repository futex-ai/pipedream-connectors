//! Identifier boundary and serde validation tests.

use serde::de::DeserializeOwned;

use crate::{
    AccountId, AppCategoryId, AppOverrideId, AppSlug, ComponentKey, ConnectToken, Cursor,
    DeployedTriggerId, ExternalUserId, OauthAppId, ProjectId, PropName, WebhookId,
    WorkflowEndpoint,
};

fn rejects<T: DeserializeOwned>(value: &str) {
    assert!(serde_json::from_value::<T>(serde_json::json!(value)).is_err());
}

#[test]
fn every_identifier_rejects_invalid_serde_input() {
    macro_rules! check {
        ($($ty:ty),+) => { $(rejects::<$ty>("bad/value"); rejects::<$ty>("");)+ };
    }
    check!(
        ProjectId,
        ExternalUserId,
        AccountId,
        DeployedTriggerId,
        OauthAppId,
        AppCategoryId,
        AppOverrideId,
        AppSlug,
        ComponentKey,
        PropName,
        WebhookId,
        Cursor,
        WorkflowEndpoint,
        ConnectToken
    );
}

#[test]
fn common_grammar_checks_all_forbidden_bytes_and_lengths() {
    for byte in 0..=u8::MAX {
        if !byte.is_ascii_graphic() || matches!(byte, b'/' | b'\\' | b'?' | b'#') {
            let value = format!("a{}b", char::from(byte));
            assert!(ExternalUserId::new(&value).is_err(), "accepted byte {byte}");
        }
    }
    for value in ["", "space here", "é", "x\n", "\u{7f}"] {
        assert!(ExternalUserId::new(value).is_err());
    }
    assert!(ExternalUserId::new("x".repeat(250)).is_ok());
    assert!(ExternalUserId::new("x".repeat(251)).is_err());
    for value in [
        ".",
        "..",
        "Future.App",
        "abc:!$&'()*+,;=@[]_~-",
        "p_endpoint",
    ] {
        assert!(ExternalUserId::new(value).is_ok());
        assert!(AppSlug::new(value).is_ok());
        assert!(WorkflowEndpoint::new(value).is_ok());
    }
    assert!(WorkflowEndpoint::new("p_/bad").is_err());
    assert!(AppCategoryId::new("appcat_demo").is_ok());
    assert!(ComponentKey::new("google_calendar.list").is_ok());
    assert!(PropName::new("account_id").is_ok());
    assert!(WebhookId::new("wh_demo").is_ok());
    assert!(Cursor::new("opaque-cursor").is_ok());
}

#[test]
fn documented_prefixes_require_an_alphanumeric_suffix() {
    macro_rules! check {
        ($ty:ty, $prefix:literal) => {
            assert!(<$ty>::new(concat!($prefix, "demo")).is_ok());
            for value in [
                $prefix.to_owned(),
                "wrong_demo".into(),
                format!("{}x-y", $prefix),
            ] {
                assert!(<$ty>::new(&value).is_err());
                rejects::<$ty>(&value);
            }
            let max = format!("{}{}", $prefix, "a".repeat(250 - $prefix.len()));
            assert!(<$ty>::new(&max).is_ok());
            assert!(<$ty>::new(format!("{max}a")).is_err());
        };
    }
    check!(ProjectId, "proj_");
    check!(AccountId, "apn_");
    check!(DeployedTriggerId, "dc_");
    assert!(DeployedTriggerId::new("hi_Interface123").is_ok());
    assert!(DeployedTriggerId::new("ti_Timer123").is_ok());
    check!(OauthAppId, "oa_");
    check!(AppCategoryId, "appcat_");
}

#[test]
fn connect_tokens_require_exactly_32_lowercase_hex_digits() {
    let valid = "ctok_0123456789abcdef0123456789abcdef";
    assert_eq!(ConnectToken::new(valid).unwrap().expose_secret(), valid);
    for invalid in ["ctok_", "wrong_0123456789abcdef0123456789abcdef"] {
        assert!(ConnectToken::new(invalid).is_err());
        rejects::<ConnectToken>(invalid);
    }
    for suffix in [
        "a".repeat(31),
        "a".repeat(33),
        "A".repeat(32),
        "g".repeat(32),
    ] {
        let value = format!("ctok_{suffix}");
        assert!(ConnectToken::new(&value).is_err());
        rejects::<ConnectToken>(&value);
    }
}
