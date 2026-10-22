//! URL boundary validation tests.

use super::{UrlRules, parse_validated};

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
