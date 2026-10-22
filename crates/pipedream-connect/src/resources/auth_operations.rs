//! Pure request builders and response identity checks for managed auth resources.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use http::{HeaderMap, HeaderName, Method};
use pipedream_connect_types::{
    Account, AccountId, AccountRef, AppSlug, CreateAccountQuery, CreateAccountRequest,
    CreateConnectTokenRequest, ExternalUserId, GetAccountRequest, ListAccountsRequest,
    ListUserAccountsRequest, ListUsersRequest, ProjectId, ValidateTokenRequest,
};
use serde::Serialize;
use url::Url;

use crate::error::{Error, InvalidRequest, Result, WireFailure};
use crate::request::{ApiPath, ApiRequest, ResponseMode};
use crate::validation::Validate;

use super::proxy::ProxyRequest;

/// Builds a Connect token creation request.
pub(super) fn create_token(
    request: &CreateConnectTokenRequest,
    project_id: &ProjectId,
) -> Result<ApiRequest> {
    let mut request = request.clone();
    request.project_id = Some(project_id.clone());
    ApiRequest::new(Method::POST, ApiPath::project(&["tokens"])).with_json(&request)
}

/// Builds a Connect token validation request.
pub(super) fn validate_token(token: &str, request: &ValidateTokenRequest) -> Result<ApiRequest> {
    ApiRequest::new(
        Method::GET,
        ApiPath::connect(&["tokens", token, "validate"]),
    )
    .with_query(request)
}

/// Builds an account list request.
pub(super) fn list_accounts(request: &ListAccountsRequest) -> Result<ApiRequest> {
    ApiRequest::new(Method::GET, ApiPath::project(&["accounts"])).with_query(request)
}

/// Builds an account retrieval request.
pub(super) fn get_account(
    account_id: &AccountId,
    request: &GetAccountRequest,
) -> Result<ApiRequest> {
    ApiRequest::new(
        Method::GET,
        ApiPath::project(&["accounts", account_id.as_str()]),
    )
    .with_query(request)
}

/// Builds an account retrieval request with its external-user partition.
pub(super) fn get_owned_account(
    account: &AccountRef,
    request: &GetAccountRequest,
) -> Result<ApiRequest> {
    get_account(&account.account_id, request).and_then(|operation| {
        operation.with_query(&OwnedAccountQuery {
            external_user_id: account.external_user_id.as_str(),
        })
    })
}

/// Builds an account creation request.
pub(super) fn create_account(
    request: &CreateAccountRequest,
    query: &CreateAccountQuery,
) -> Result<ApiRequest> {
    ApiRequest::new(Method::POST, ApiPath::project(&["accounts"]))
        .with_query(query)?
        .with_json(request)
}

/// Builds an account deletion request.
pub(super) fn delete_account(account_id: &AccountId) -> ApiRequest {
    ApiRequest::new(
        Method::DELETE,
        ApiPath::project(&["accounts", account_id.as_str()]),
    )
}

/// Builds an account deletion request with its external-user partition.
pub(super) fn delete_owned_account(account: &AccountRef) -> Result<ApiRequest> {
    delete_account(&account.account_id).with_query(&OwnedAccountQuery {
        external_user_id: account.external_user_id.as_str(),
    })
}

/// Builds deletion of every account owned by an app in the project.
pub(super) fn delete_accounts_by_app(app: &AppSlug) -> ApiRequest {
    ApiRequest::new(
        Method::DELETE,
        ApiPath::project(&["apps", app.as_str(), "accounts"]),
    )
}

/// Builds an external-user list request.
pub(super) fn list_users(request: &ListUsersRequest) -> Result<ApiRequest> {
    ApiRequest::new(Method::GET, ApiPath::project(&["users"])).with_query(request)
}

/// Builds deletion of an external user and its resources.
pub(super) fn delete_user(external_user_id: &str) -> ApiRequest {
    ApiRequest::new(
        Method::DELETE,
        ApiPath::project(&["users", external_user_id]),
    )
}

/// Builds a list request for one external user's accounts.
pub(super) fn list_user_accounts(
    external_user_id: &str,
    request: &ListUserAccountsRequest,
) -> Result<ApiRequest> {
    ApiRequest::new(
        Method::GET,
        ApiPath::project(&["users", external_user_id, "accounts"]),
    )
    .with_query(request)
}

/// Builds a proxy request after validating its target and caller headers.
pub(super) fn proxy(request: ProxyRequest) -> Result<ApiRequest> {
    validate_target(&request.target)?;
    let target = URL_SAFE_NO_PAD.encode(request.target.as_bytes());
    let headers = proxy_headers(&request.headers)?;
    let query = ProxyQuery {
        external_user_id: request.account.external_user_id.as_str(),
        account_id: request.account.account_id.as_str(),
    };
    let operation = ApiRequest::new(
        request.method.into(),
        ApiPath::project(&["proxy", target.as_str()]),
    )
    .with_query(&query)?
    .with_headers(headers)
    .with_response_mode(ResponseMode::PassThrough);
    Ok(match request.body {
        Some(body) => operation.with_bytes(body),
        None => operation,
    })
}

#[derive(Serialize)]
struct ProxyQuery<'a> {
    external_user_id: &'a str,
    account_id: &'a str,
}

#[derive(Serialize)]
struct OwnedAccountQuery<'a> {
    external_user_id: &'a str,
}

impl Validate for OwnedAccountQuery<'_> {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

impl Validate for ProxyQuery<'_> {
    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

/// Checks a returned account against the requested path identity.
pub(super) fn check_account(response: Account, requested: &AccountId) -> Result<Account> {
    if &response.id == requested {
        Ok(response)
    } else {
        Err(Error::internal(WireFailure::IdentityMismatch))
    }
}

/// Checks every returned account against one external-user partition.
pub(super) fn check_owned_accounts(
    accounts: &[Account],
    external_user_id: &ExternalUserId,
    account_id: Option<&AccountId>,
) -> Result<()> {
    if accounts
        .iter()
        .any(|account| account.external_id.as_ref() != Some(external_user_id))
    {
        return Err(Error::NotFound { message: None });
    }
    if account_id.is_some_and(|id| !accounts.iter().any(|account| &account.id == id)) {
        return Err(Error::NotFound { message: None });
    }
    Ok(())
}

/// Validates an absolute HTTPS target or a single relative path.
///
/// Absolute targets keep their host and reject credentials and fragments.
/// Relative targets must start with one slash and cannot contain a fragment.
fn validate_target(target: &str) -> Result<()> {
    if target.is_empty() || target.trim() != target || target.chars().any(char::is_control) {
        return Err(Error::InvalidRequest(InvalidRequest::ProxyTarget));
    }
    if let Ok(url) = Url::parse(target) {
        if url.scheme() != "https"
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
        {
            return Err(Error::InvalidRequest(InvalidRequest::ProxyTarget));
        }
        return Ok(());
    }
    if target.starts_with('/') && !target.starts_with("//") && !target.contains('#') {
        return Ok(());
    }
    Err(Error::InvalidRequest(InvalidRequest::ProxyTarget))
}

/// Admits caller headers and prefixes them for Connect forwarding.
///
/// Reserved names are rejected before authentication. Repeated values keep
/// their original order and sensitivity flag.
fn proxy_headers(headers: &HeaderMap) -> Result<HeaderMap> {
    let mut forwarded = HeaderMap::new();
    for name in headers.keys() {
        let lower = name.as_str().to_ascii_lowercase();
        if restricted_header(&lower) {
            return Err(Error::InvalidRequest(InvalidRequest::ProxyHeader));
        }
        let forwarded_name = format!("x-pd-proxy-{lower}");
        let forwarded_name = match HeaderName::from_bytes(forwarded_name.as_bytes()) {
            Ok(value) => value,
            Err(_) => return Err(Error::InvalidRequest(InvalidRequest::ProxyHeader)),
        };
        for value in headers.get_all(name).iter() {
            forwarded.append(forwarded_name.clone(), value.clone());
        }
    }
    Ok(forwarded)
}

fn restricted_header(name: &str) -> bool {
    matches!(
        name,
        "accept-encoding"
            | "access-control-request-headers"
            | "access-control-request-method"
            | "connection"
            | "content-length"
            | "cookie"
            | "date"
            | "dnt"
            | "expect"
            | "host"
            | "keep-alive"
            | "origin"
            | "permissions-policy"
            | "referer"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
            | "via"
    ) || name.starts_with("proxy-")
        || name.starts_with("sec-")
        || name.starts_with("x-pd-")
}
