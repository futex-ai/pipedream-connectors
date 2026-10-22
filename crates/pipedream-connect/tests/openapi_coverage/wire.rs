use http::Method;
use pipedream_connect::transport::HttpRequest;
use url::Url;

#[derive(Clone, Copy)]
pub struct Wire {
    id: &'static str,
    method: &'static str,
    url: &'static str,
    body: Option<&'static [u8]>,
    accept: bool,
    content_type: bool,
    authenticated: bool,
}

impl Wire {
    pub fn for_operation(id: &str) -> Option<Self> {
        let row = match id {
            "listAppCategories" => api("listAppCategories", "GET", "/v1/connect/app_categories", None),
            "retrieveAppCategory" => api("retrieveAppCategory", "GET", "/v1/connect/app_categories/appcat_Test123", None),
            "listApps" => api("listApps", "GET", "/v1/connect/apps?limit=100", None),
            "retrieveApp" => api("retrieveApp", "GET", "/v1/connect/apps/google_calendar", None),
            "listAccounts" => api("listAccounts", "GET", "/v1/connect/proj_Test123/accounts?include_credentials=false&limit=100", None),
            "createAccount" => api("createAccount", "POST", "/v1/connect/proj_Test123/accounts", Some(br#"{"app_slug":"google_calendar","cfmap_json":"{}","connect_token":"ctok_00000000000000000000000000000000"}"#)),
            "retrieveAccount" => api("retrieveAccount", "GET", "/v1/connect/proj_Test123/accounts/apn_Account123?include_credentials=false", None),
            "deleteAccount" => api("deleteAccount", "DELETE", "/v1/connect/proj_Test123/accounts/apn_Account123", None),
            "deleteAccountByApp" => api("deleteAccountByApp", "DELETE", "/v1/connect/proj_Test123/apps/google_calendar/accounts", None),
            "deleteUserExternalUser" => api("deleteUserExternalUser", "DELETE", "/v1/connect/proj_Test123/users/customer_1", None),
            "listUserAccounts" => api("listUserAccounts", "GET", "/v1/connect/proj_Test123/users/customer_1/accounts?include_credentials=false", None),
            "listAppOverrides" => api("listAppOverrides", "GET", "/v1/connect/proj_Test123/app_overrides?limit=100", None),
            "createAppOverride" => api("createAppOverride", "POST", "/v1/connect/proj_Test123/app_overrides", Some(br#"{"app":"google_calendar","name":"Override"}"#)),
            "retrieveAppOverride" => api("retrieveAppOverride", "GET", "/v1/connect/proj_Test123/app_overrides/ao_Test123", None),
            "updateAppOverride" => api("updateAppOverride", "PUT", "/v1/connect/proj_Test123/app_overrides/ao_Test123", Some(br#"{"name":"Updated"}"#)),
            "deleteAppOverride" => api("deleteAppOverride", "DELETE", "/v1/connect/proj_Test123/app_overrides/ao_Test123", None),
            "listComponents" => api("listComponents", "GET", "/v1/connect/proj_Test123/components?limit=100", None),
            "retrieveComponent" => api("retrieveComponent", "GET", "/v1/connect/proj_Test123/components/google_calendar-list-events?version=1.0.0", None),
            "configureComponentProp" => api("configureComponentProp", "POST", "/v1/connect/proj_Test123/components/configure", Some(br#"{"id":"google_calendar-list-events","external_user_id":"customer_1","prop_name":"CalendarID"}"#)),
            "reloadComponentProps" => api("reloadComponentProps", "POST", "/v1/connect/proj_Test123/components/props", Some(br#"{"id":"google_calendar-list-events","external_user_id":"customer_1"}"#)),
            "listActions" => api("listActions", "GET", "/v1/connect/proj_Test123/actions?limit=100", None),
            "retrieveAction" => api("retrieveAction", "GET", "/v1/connect/proj_Test123/actions/google_calendar-list-events?version=1.0.0", None),
            "configureActionProp" => api("configureActionProp", "POST", "/v1/connect/proj_Test123/actions/configure", Some(br#"{"id":"google_calendar-list-events","external_user_id":"customer_1","prop_name":"CalendarID"}"#)),
            "reloadActionProps" => api("reloadActionProps", "POST", "/v1/connect/proj_Test123/actions/props", Some(br#"{"id":"google_calendar-list-events","external_user_id":"customer_1"}"#)),
            "listTriggers" => api("listTriggers", "GET", "/v1/connect/proj_Test123/triggers?limit=100", None),
            "retrieveTrigger" => api("retrieveTrigger", "GET", "/v1/connect/proj_Test123/triggers/google_calendar-list-events?version=1.0.0", None),
            "configureTriggerProp" => api("configureTriggerProp", "POST", "/v1/connect/proj_Test123/triggers/configure", Some(br#"{"id":"google_calendar-list-events","external_user_id":"customer_1","prop_name":"CalendarID"}"#)),
            "reloadTriggerProps" => api("reloadTriggerProps", "POST", "/v1/connect/proj_Test123/triggers/props", Some(br#"{"id":"google_calendar-list-events","external_user_id":"customer_1"}"#)),
            "runAction" => api("runAction", "POST", "/v1/connect/proj_Test123/actions/run", Some(br#"{"id":"google_calendar-list-events","external_user_id":"customer_1"}"#)),
            "deployTrigger" => api("deployTrigger", "POST", "/v1/connect/proj_Test123/triggers/deploy", Some(br#"{"id":"google_calendar-list-events","external_user_id":"customer_1"}"#)),
            "listDeployedTriggers" => api("listDeployedTriggers", "GET", "/v1/connect/proj_Test123/deployed-triggers?external_user_id=customer_1&limit=100", None),
            "retrieveDeployedTrigger" => api("retrieveDeployedTrigger", "GET", "/v1/connect/proj_Test123/deployed-triggers/dc_Trigger123?external_user_id=customer_1", None),
            "updateDeployedTrigger" => api("updateDeployedTrigger", "PUT", "/v1/connect/proj_Test123/deployed-triggers/dc_Trigger123?external_user_id=customer_1", Some(br#"{}"#)),
            "deleteDeployedTrigger" => api("deleteDeployedTrigger", "DELETE", "/v1/connect/proj_Test123/deployed-triggers/dc_Trigger123?external_user_id=customer_1", None),
            "listDeployedTriggerEvents" => api("listDeployedTriggerEvents", "GET", "/v1/connect/proj_Test123/deployed-triggers/dc_Trigger123/events?external_user_id=customer_1&n=10", None),
            "listDeployedTriggerWorkflows" => api("listDeployedTriggerWorkflows", "GET", "/v1/connect/proj_Test123/deployed-triggers/dc_Trigger123/pipelines?external_user_id=customer_1", None),
            "updateDeployedTriggerWorkflows" => api("updateDeployedTriggerWorkflows", "PUT", "/v1/connect/proj_Test123/deployed-triggers/dc_Trigger123/pipelines?external_user_id=customer_1", Some(br#"{"workflow_ids":["p_workflow1"]}"#)),
            "listDeployedTriggerWebhooks" => api("listDeployedTriggerWebhooks", "GET", "/v1/connect/proj_Test123/deployed-triggers/dc_Trigger123/webhooks?external_user_id=customer_1", None),
            "updateDeployedTriggerWebhooks" => api("updateDeployedTriggerWebhooks", "PUT", "/v1/connect/proj_Test123/deployed-triggers/dc_Trigger123/webhooks?external_user_id=customer_1", Some(br#"{"webhook_urls":["https://example.com/events"]}"#)),
            "retrieveDeployedTriggerWebhook" => api("retrieveDeployedTriggerWebhook", "GET", "/v1/connect/proj_Test123/deployed-triggers/dc_Trigger123/webhooks/wh_Webhook123?external_user_id=customer_1", None),
            "regenerateDeployedTriggerWebhookSigningKey" => api("regenerateDeployedTriggerWebhookSigningKey", "POST", "/v1/connect/proj_Test123/deployed-triggers/dc_Trigger123/webhooks/wh_Webhook123/regenerate_signing_key?external_user_id=customer_1", None),
            "listOauthApps" => api("listOauthApps", "GET", "/v1/connect/proj_Test123/oauth_apps?limit=100", None),
            "createOauthApp" => api("createOauthApp", "POST", "/v1/connect/proj_Test123/oauth_apps", Some(br#"{"app":"google_calendar","client_id":"upstream-client","client_secret":"upstream-secret"}"#)),
            "retrieveOauthApp" => api("retrieveOauthApp", "GET", "/v1/connect/proj_Test123/oauth_apps/oa_Test123", None),
            "updateOauthApp" => api("updateOauthApp", "PUT", "/v1/connect/proj_Test123/oauth_apps/oa_Test123", Some(br#"{}"#)),
            "deleteOauthApp" => api("deleteOauthApp", "DELETE", "/v1/connect/proj_Test123/oauth_apps/oa_Test123", None),
            "retrieveProjectEnvironmentWebhook" => api("retrieveProjectEnvironmentWebhook", "GET", "/v1/connect/proj_Test123/webhook", None),
            "updateProjectEnvironmentWebhook" => api("updateProjectEnvironmentWebhook", "PUT", "/v1/connect/proj_Test123/webhook", Some(br#"{"url":"https://example.com/events"}"#)),
            "deleteProjectEnvironmentWebhook" => api("deleteProjectEnvironmentWebhook", "DELETE", "/v1/connect/proj_Test123/webhook", None),
            "regenerateProjectEnvironmentWebhookSigningKey" => api("regenerateProjectEnvironmentWebhookSigningKey", "POST", "/v1/connect/proj_Test123/webhook/regenerate_signing_key", None),
            "listProjects" => api("listProjects", "GET", "/v1/connect/projects?limit=100", None),
            "createProject" => api("createProject", "POST", "/v1/connect/projects", Some(br#"{"name":"Demo"}"#)),
            "retrieveProject" => api("retrieveProject", "GET", "/v1/connect/projects/proj_Test123", None),
            "deleteProject" => api("deleteProject", "DELETE", "/v1/connect/projects/proj_Test123", None),
            "updateProject" => api("updateProject", "PATCH", "/v1/connect/projects/proj_Test123", Some(br#"{"name":"Updated"}"#)),
            "updateProjectLogo" => api("updateProjectLogo", "POST", "/v1/connect/projects/proj_Test123/logo", Some(br#"{"logo":"data:image/png;base64,AA=="}"#)),
            "retrieveProjectInfo" => api("retrieveProjectInfo", "GET", "/v1/connect/proj_Test123/projects/info", None),
            "downloadFileStashFile" => special("downloadFileStashFile", "GET", "/v1/connect/proj_Test123/file_stash/download?s3_key=stash-key", None),
            "retrieveProxy" => proxy("retrieveProxy", "GET", "/v1/connect/proj_Test123/proxy/L3YxL2V2ZW50cw?account_id=apn_Account123&external_user_id=customer_1", None),
            "postProxy" => proxy("postProxy", "POST", "/v1/connect/proj_Test123/proxy/L3YxL2V2ZW50cw?account_id=apn_Account123&external_user_id=customer_1", Some(b"body")),
            "putProxy" => proxy("putProxy", "PUT", "/v1/connect/proj_Test123/proxy/L3YxL2V2ZW50cw?account_id=apn_Account123&external_user_id=customer_1", Some(b"body")),
            "deleteProxy" => proxy("deleteProxy", "DELETE", "/v1/connect/proj_Test123/proxy/L3YxL2V2ZW50cw?account_id=apn_Account123&external_user_id=customer_1", None),
            "patchProxy" => proxy("patchProxy", "PATCH", "/v1/connect/proj_Test123/proxy/L3YxL2V2ZW50cw?account_id=apn_Account123&external_user_id=customer_1", Some(b"body")),
            "createToken" => api("createToken", "POST", "/v1/connect/proj_Test123/tokens", Some(br#"{"project_id":"proj_Test123","external_user_id":"customer_1","expires_in":14400}"#)),
            "validateToken" => api("validateToken", "GET", "/v1/connect/tokens/ctok_00000000000000000000000000000000/validate?app_id=google_calendar", None),
            "listUsages" => api("listUsages", "GET", "/v1/connect/usage?end_ts=2000&start_ts=1000", None),
            "listUsers" => api("listUsers", "GET", "/v1/connect/proj_Test123/users?limit=100", None),
            "createOauthToken" => oauth("createOauthToken", "POST", "/v1/oauth/token", Some(br#"{"grant_type":"client_credentials","client_id":"client-id","client_secret":"client-secret"}"#)),
            _ => return None,
        };
        Some(row)
    }

    pub fn assert_request(&self, request: &HttpRequest, openapi_method: &str, template: &str) {
        assert_eq!(
            self.method, openapi_method,
            "wrong golden method for {}",
            self.id
        );
        assert_eq!(
            request.method,
            Method::from_bytes(self.method.as_bytes()).unwrap(),
            "wrong method for {}",
            self.id
        );
        let actual_url = request.url.expose_secret();
        assert_eq!(
            actual_url,
            format!("https://api.pipedream.com{}", self.url),
            "wrong URL for {}",
            self.id
        );
        let parsed_url = Url::parse(actual_url).expect("request URL");
        assert_template(template, parsed_url.path(), self.id);
        assert_eq!(
            request.body.as_deref(),
            self.body,
            "wrong body for {}",
            self.id
        );
        assert_header(
            request,
            "authorization",
            self.authenticated.then_some("Bearer access-token"),
            self.id,
        );
        assert_header(
            request,
            "x-pd-environment",
            self.authenticated.then_some("development"),
            self.id,
        );
        assert_header(
            request,
            "content-type",
            self.content_type.then_some("application/json"),
            self.id,
        );
        assert_header(
            request,
            "accept",
            self.accept.then_some("application/json"),
            self.id,
        );
        assert_header(request, "x-pd-rate-limit", None, self.id);
    }
}

fn api(
    id: &'static str,
    method: &'static str,
    url: &'static str,
    body: Option<&'static [u8]>,
) -> Wire {
    Wire {
        id,
        method,
        url,
        body,
        accept: true,
        content_type: body.is_some(),
        authenticated: true,
    }
}

fn special(
    id: &'static str,
    method: &'static str,
    url: &'static str,
    body: Option<&'static [u8]>,
) -> Wire {
    Wire {
        id,
        method,
        url,
        body,
        accept: false,
        content_type: false,
        authenticated: true,
    }
}

fn proxy(
    id: &'static str,
    method: &'static str,
    url: &'static str,
    body: Option<&'static [u8]>,
) -> Wire {
    Wire {
        id,
        method,
        url,
        body,
        accept: false,
        content_type: false,
        authenticated: true,
    }
}

fn oauth(
    id: &'static str,
    method: &'static str,
    url: &'static str,
    body: Option<&'static [u8]>,
) -> Wire {
    Wire {
        id,
        method,
        url,
        body,
        accept: true,
        content_type: body.is_some(),
        authenticated: false,
    }
}

fn assert_header(request: &HttpRequest, name: &str, expected: Option<&str>, operation: &str) {
    let actual = request
        .headers
        .get(name)
        .and_then(|value| value.to_str().ok());
    assert_eq!(actual, expected, "wrong {name} header for {operation}");
}

fn assert_template(template: &str, path: &str, operation: &str) {
    let expected: Vec<_> = template
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();
    let actual: Vec<_> = path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();
    assert_eq!(
        expected.len(),
        actual.len(),
        "wrong path segment count for {operation}"
    );
    for (expected, actual) in expected.iter().zip(actual) {
        if !expected.starts_with('{') {
            assert_eq!(expected, &actual, "wrong path segment for {operation}");
        } else {
            assert!(!actual.is_empty(), "empty path parameter for {operation}");
        }
    }
}
