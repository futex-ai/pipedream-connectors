//! Pure request builders and response identity checks for managed auth resources.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use http::{HeaderMap, HeaderName, HeaderValue, Method};
use pipedream_connect_types::{
    Account, AccountId, AccountRef, AppSlug, CreateAccountRequest, CreateConnectTokenRequest,
    GetAccountRequest, ListAccountsRequest, ListUserAccountsRequest, ListUsersRequest,
    ValidateTokenRequest,
};
use serde::Serialize;
use url::Url;

use crate::error::{Error, InvalidRequest, Result, WireFailure};
use crate::request::{ApiPath, ApiRequest, ResponseMode};
use crate::validation::Validate;

use super::proxy::ProxyRequest;

/// Builds a Connect token creation request.
pub(super) fn create_token(request: &CreateConnectTokenRequest) -> Result<ApiRequest> {
    ApiRequest::new(Method::POST, ApiPath::project(&["tokens"])).with_json(request)
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
pub(super) fn create_account(request: &CreateAccountRequest) -> Result<ApiRequest> {
    ApiRequest::new(Method::POST, ApiPath::project(&["accounts"])).with_json(request)
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
pub(super) fn proxy(request: &ProxyRequest) -> Result<ApiRequest> {
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
    Ok(match &request.body {
        Some(body) => operation.with_bytes(body.clone()),
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

/// Checks both account and external-user ownership.
pub(super) fn check_account_owner(response: Account, account: &AccountRef) -> Result<Account> {
    if response.id == account.account_id
        && response
            .external_id
            .as_ref()
            .is_some_and(|external_id| external_id == &account.external_user_id)
    {
        Ok(response)
    } else {
        Err(Error::internal(WireFailure::IdentityMismatch))
    }
}

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

fn proxy_headers(headers: &HeaderMap) -> Result<HeaderMap> {
    let mut forwarded = HeaderMap::new();
    for name in headers.keys() {
        let lower = name.as_str().to_ascii_lowercase();
        if restricted_header(&lower) {
            return Err(Error::InvalidRequest(InvalidRequest::ProxyHeader));
        }
        let forwarded_name = format!("x-pd-proxy-{lower}");
        let forwarded_name = HeaderName::from_bytes(forwarded_name.as_bytes())
            .map_err(|_| Error::InvalidRequest(InvalidRequest::ProxyHeader))?;
        for value in headers.get_all(name).iter() {
            let value = HeaderValue::from_bytes(value.as_bytes())
                .map_err(|_| Error::InvalidRequest(InvalidRequest::ProxyHeader))?;
            forwarded.append(forwarded_name.clone(), value);
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
