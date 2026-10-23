//! Client calls for catalog and execution coverage.

use pipedream_connect::PipedreamClient;
use pipedream_connect_types::{
    AppCategoryId, AppOverrideId, AppSlug, ComponentKey, CreateAccountQuery,
    CreateAppOverrideRequest, ExternalUserId, GetAccountRequest, GetComponentRequest,
    ListAccountsRequest, ListUserAccountsRequest, UpdateAppOverrideRequest,
};

use super::cases::{
    account_id, account_request, configure_request, deploy_request, reload_request, run_request,
};

pub async fn invoke(client: &PipedreamClient, id: &str) -> bool {
    let user = || ExternalUserId::new("customer_1").expect("external user ID");
    let component = || ComponentKey::new("google_calendar-list-events").expect("component key");
    match id {
        "listAppCategories" => {
            let _ = client.app_categories().list().await;
        }
        "retrieveAppCategory" => {
            let _ = client
                .app_categories()
                .get(AppCategoryId::new("appcat_Test123").unwrap())
                .await;
        }
        "listApps" => {
            let _ = client.apps().list(Default::default()).await;
        }
        "retrieveApp" => {
            let _ = client
                .apps()
                .get(AppSlug::new("google_calendar").unwrap())
                .await;
        }
        "listAccounts" => {
            let _ = client.accounts().list(ListAccountsRequest::default()).await;
        }
        "createAccount" => {
            let request = account_request();
            let _ = client
                .accounts()
                .create(request, CreateAccountQuery::default())
                .await;
        }
        "retrieveAccount" => {
            let _ = client
                .accounts()
                .get(account_id(), GetAccountRequest::default())
                .await;
        }
        "deleteAccount" => {
            let _ = client.accounts().delete(account_id()).await;
        }
        "deleteAccountByApp" => {
            let _ = client
                .accounts()
                .delete_by_app(AppSlug::new("google_calendar").unwrap())
                .await;
        }
        "deleteUserExternalUser" => {
            let _ = client.users().delete(user()).await;
        }
        "listUserAccounts" => {
            let _ = client
                .users()
                .list_accounts(user(), ListUserAccountsRequest::default())
                .await;
        }
        "listAppOverrides" => {
            let _ = client.app_overrides().list(Default::default()).await;
        }
        "createAppOverride" => {
            let _ = client
                .app_overrides()
                .create(CreateAppOverrideRequest {
                    app: "google_calendar".into(),
                    name: "Override".into(),
                    oauth_app_id: None,
                    cfmap: None,
                })
                .await;
        }
        "retrieveAppOverride" => {
            let _ = client
                .app_overrides()
                .get(AppOverrideId::new("ao_Test123").unwrap())
                .await;
        }
        "updateAppOverride" => {
            let _ = client
                .app_overrides()
                .update(
                    AppOverrideId::new("ao_Test123").unwrap(),
                    UpdateAppOverrideRequest {
                        name: Some("Updated".into()),
                        ..Default::default()
                    },
                )
                .await;
        }
        "deleteAppOverride" => {
            let _ = client
                .app_overrides()
                .delete(AppOverrideId::new("ao_Test123").unwrap())
                .await;
        }
        "listComponents" => {
            let _ = client.components().list(Default::default()).await;
        }
        "retrieveComponent" => {
            let _ = client
                .components()
                .get(
                    component(),
                    GetComponentRequest {
                        version: Some("1.0.0".into()),
                    },
                )
                .await;
        }
        "configureComponentProp" => {
            let _ = client.components().configure(configure_request()).await;
        }
        "reloadComponentProps" => {
            let _ = client.components().reload_props(reload_request()).await;
        }
        "listActions" => {
            let _ = client.actions().list(Default::default()).await;
        }
        "retrieveAction" => {
            let _ = client
                .actions()
                .get(
                    component(),
                    GetComponentRequest {
                        version: Some("1.0.0".into()),
                    },
                )
                .await;
        }
        "configureActionProp" => {
            let _ = client.actions().configure(configure_request()).await;
        }
        "reloadActionProps" => {
            let _ = client.actions().reload_props(reload_request()).await;
        }
        "listTriggers" => {
            let _ = client.triggers().list(Default::default()).await;
        }
        "retrieveTrigger" => {
            let _ = client
                .triggers()
                .get(
                    component(),
                    GetComponentRequest {
                        version: Some("1.0.0".into()),
                    },
                )
                .await;
        }
        "configureTriggerProp" => {
            let _ = client.triggers().configure(configure_request()).await;
        }
        "reloadTriggerProps" => {
            let _ = client.triggers().reload_props(reload_request()).await;
        }
        "runAction" => {
            let _ = client.actions().run(run_request()).await;
        }
        "deployTrigger" => {
            let _ = client.triggers().deploy(deploy_request()).await;
        }
        "createOauthToken" => return false,
        _ => return false,
    }
    true
}
