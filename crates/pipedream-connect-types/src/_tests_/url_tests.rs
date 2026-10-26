//! URL boundary validation tests.

use super::{UrlRules, is_valid, parse_validated};

#[test]
fn workflow_endpoint_allows_at_signs_in_a_query() {
    assert!(
        parse_validated(
            "https://eo1.m.pipedream.net?email=a@b.example",
            UrlRules::WorkflowEndpoint,
        )
        .is_some()
    );
}

#[test]
fn api_origin_still_rejects_a_query_with_an_at_sign() {
    assert!(
        parse_validated(
            "https://eo1.m.pipedream.net?email=a@b.example",
            UrlRules::ApiOrigin,
        )
        .is_none()
    );
}

#[test]
fn api_origin_still_accepts_a_root_slash() {
    assert!(parse_validated("https://eo1.m.pipedream.net/", UrlRules::ApiOrigin).is_some());
}

#[test]
fn both_rules_reject_at_signs_in_the_authority() {
    for rules in [UrlRules::ApiOrigin, UrlRules::WorkflowEndpoint] {
        assert!(parse_validated("https://user@example.com", rules).is_none());
    }
}

#[test]
fn proxy_targets_reject_backslashes_and_raw_authority_at_signs() {
    assert!(!is_valid("/\\evil.example/api", UrlRules::ProxyTarget));
    assert!(!is_valid(
        "https://user@example.com/path",
        UrlRules::ProxyTarget
    ));
}

#[test]
fn proxy_targets_keep_relative_paths_and_query_at_signs() {
    assert!(is_valid(
        "/calendar/events?owner=a@b.example",
        UrlRules::ProxyTarget
    ));
    assert!(is_valid(
        "https://example.com/calendar?owner=a@b.example",
        UrlRules::ProxyTarget
    ));
}
