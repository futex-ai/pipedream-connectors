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
}

/// Parses a URL when it satisfies the selected boundary rules.
pub fn parse_validated(value: &str, rules: UrlRules) -> Option<Url> {
    let url = Url::parse(value).ok()?;
    match rules {
        UrlRules::ApiOrigin => {
            let bare_authority = value.split_once("://").is_some_and(|(_, rest)| {
                let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
                !authority.contains('@') && path.is_empty()
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
            let authority_has_userinfo = value.split_once("://").is_some_and(|(_, rest)| {
                rest.split('/')
                    .next()
                    .is_some_and(|authority| authority.contains('@'))
            });
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
    }
    Some(url)
}
