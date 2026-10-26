//! Shared URL validation rules used by client and model boundaries.

use url::Url;

/// Explicit URL rules for each Connect URL boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UrlRules {
    /// A bare HTTPS API origin without path, query, fragment, or credentials.
    ApiOrigin,
    /// An HTTPS workflow endpoint without credentials, fragments, controls, or backslashes.
    WorkflowEndpoint,
    /// An absolute HTTPS URL with a host.
    HttpsHost,
    /// An absolute HTTPS URL or a single relative path for proxy forwarding.
    /// `parse_validated` checks only absolute proxy targets.
    ProxyTarget,
}

/// Parses an absolute URL when it satisfies the selected boundary rules.
/// For `ProxyTarget`, relative paths are checked only by `is_valid`.
pub fn parse_validated(value: &str, rules: UrlRules) -> Option<Url> {
    let url = Url::parse(value).ok()?;
    match rules {
        UrlRules::ApiOrigin => {
            let bare_authority = authority(value).is_some_and(|(authority, remainder)| {
                !authority.contains('@') && matches!(remainder, "" | "/")
            });
            if value.trim() != value
                || value.chars().any(char::is_control)
                || value.contains('\\')
                || !bare_authority
                || url.scheme() != "https"
                || url.host_str().is_none()
                || !url.username().is_empty()
                || url.password().is_some()
                || url.path() != "/"
                || url.query().is_some()
                || url.fragment().is_some()
            {
                return None;
            }
        }
        UrlRules::WorkflowEndpoint => {
            let authority_has_userinfo =
                authority(value).is_some_and(|(authority, _)| authority.contains('@'));
            if value.trim() != value
                || value.chars().any(char::is_control)
                || value.contains('\\')
                || url.scheme() != "https"
                || url.host_str().is_none()
                || authority_has_userinfo
                || !url.username().is_empty()
                || url.password().is_some()
                || url.fragment().is_some()
            {
                return None;
            }
        }
        UrlRules::HttpsHost => {
            if url.scheme() != "https" || url.host_str().is_none() {
                return None;
            }
        }
        UrlRules::ProxyTarget => {
            if value.trim() != value
                || value.chars().any(char::is_control)
                || value.contains('\\')
                || url.scheme() != "https"
                || url.host_str().is_none()
                || authority_contains_at(value)
                || !url.username().is_empty()
                || url.password().is_some()
                || url.fragment().is_some()
            {
                return None;
            }
        }
    }
    Some(url)
}

/// Checks an absolute URL or relative path against the selected URL rules.
/// `parse_validated` checks only absolute proxy targets, so this helper also
/// validates single-slash relative proxy paths.
pub fn is_valid(value: &str, rules: UrlRules) -> bool {
    if rules != UrlRules::ProxyTarget {
        return parse_validated(value, rules).is_some();
    }
    parse_validated(value, rules).is_some()
        || (value.trim() == value
            && !value.is_empty()
            && !value.chars().any(char::is_control)
            && !value.contains('\\')
            && value.starts_with('/')
            && !value.starts_with("//")
            && !value.contains('#'))
}

fn authority(value: &str) -> Option<(&str, &str)> {
    let (_, rest) = value.split_once("://")?;
    let boundary = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    Some(rest.split_at(boundary))
}

fn authority_contains_at(value: &str) -> bool {
    authority(value).is_some_and(|(authority, _)| authority.contains('@'))
}

#[cfg(test)]
#[path = "_tests_/url_tests.rs"]
mod url_tests;
