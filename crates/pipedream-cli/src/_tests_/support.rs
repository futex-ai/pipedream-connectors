//! A harness that runs the CLI against one mock for every process boundary.

use std::ffi::OsString;
use std::sync::Arc;

use pipedream_connect::resources::accounts::AccountsApi;
use pipedream_connect::resources::actions::ActionsApi;
use pipedream_connect::resources::app_categories::AppCategoriesApi;
use pipedream_connect::resources::apps::AppsApi;
use pipedream_connect::resources::components::ComponentsApi;
use pipedream_connect::resources::deployed_triggers::DeployedTriggersApi;
use pipedream_connect::resources::project_webhooks::ProjectWebhooksApi;
use pipedream_connect::resources::projects::ProjectsApi;
use pipedream_connect::resources::proxy::ProxyApi;
use pipedream_connect::resources::tokens::TokensApi;
use pipedream_connect::resources::triggers::TriggersApi;
use pipedream_connect::resources::usage::UsageApi;
use pipedream_connect::resources::users::UsersApi;
use pipedream_connect::resources::{ConnectResources, ConnectResourcesMock};
use serde::Serialize;
use unimock::{Clause, MockFn, Unimock, matching};

use crate::app::{App, Application};
use crate::connector::ConnectMock;
use crate::report::exit::Exit;
use crate::settings::Setting;
use crate::terminal::Terminal;
use crate::variables::GetVariableMock;

/// The client secret in [`Env::CONFIGURED`]. No output may show it.
pub(crate) const SECRET: &str = "client-secret-value";

/// The captured result of one CLI run.
#[derive(Debug)]
pub(crate) struct Ran {
    /// The exit status.
    pub(crate) exit: Exit,
    /// Everything written to standard output.
    pub(crate) stdout: String,
    /// Everything written to standard error.
    pub(crate) stderr: String,
}

/// Runs the CLI with `args`. One mock serves the variables, inputs, and connector.
pub(crate) async fn run(args: &[&str], clauses: impl Clause) -> Ran {
    let mock = Arc::new(Unimock::new(clauses));
    let app = App::new(mock.clone(), mock.clone(), mock);
    let mut argv = vec![OsString::from("pipedream-cli")];
    argv.extend(args.iter().map(OsString::from));
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let exit = app
        .run(argv, &mut Terminal::new(&mut stdout, &mut stderr))
        .await;
    Ran {
        exit,
        stdout: String::from_utf8(stdout).unwrap(),
        stderr: String::from_utf8(stderr).unwrap(),
    }
}

/// Raw values of the configuration variables.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Env {
    /// `PIPEDREAM_CLIENT_ID`.
    pub(crate) client_id: Option<&'static str>,
    /// `PIPEDREAM_CLIENT_SECRET`.
    pub(crate) client_secret: Option<&'static str>,
    /// `PIPEDREAM_PROJECT_ID`.
    pub(crate) project_id: Option<&'static str>,
    /// `PIPEDREAM_ENVIRONMENT`.
    pub(crate) environment: Option<&'static str>,
    /// `PIPEDREAM_API_ORIGIN`.
    pub(crate) api_origin: Option<&'static str>,
}

impl Env {
    /// A development project with valid client credentials.
    pub(crate) const CONFIGURED: Self = Self {
        client_id: Some("client-id-value"),
        client_secret: Some(SECRET),
        project_id: Some("proj_Example123"),
        environment: Some("development"),
        api_origin: None,
    };

    /// No configuration variable is set.
    pub(crate) const EMPTY: Self = Self {
        client_id: None,
        client_secret: None,
        project_id: None,
        environment: None,
        api_origin: None,
    };

    /// Serves these values for every variable read.
    pub(crate) fn clauses(self) -> impl Clause {
        (
            GetVariableMock
                .each_call(matching!(Setting::ClientId))
                .returns(self.client_id.map(OsString::from)),
            GetVariableMock
                .each_call(matching!(Setting::ClientSecret))
                .returns(self.client_secret.map(OsString::from)),
            GetVariableMock
                .each_call(matching!(Setting::ProjectId))
                .returns(self.project_id.map(OsString::from)),
            GetVariableMock
                .each_call(matching!(Setting::Environment))
                .returns(self.environment.map(OsString::from)),
            GetVariableMock
                .each_call(matching!(Setting::ApiOrigin))
                .returns(self.api_origin.map(OsString::from)),
        )
    }
}

/// Configures the CLI and connects it to resources that the same mock serves.
pub(crate) fn connected() -> impl Clause {
    (Env::CONFIGURED.clauses(), connects())
}

/// Connects the CLI to resources that the same mock serves.
///
/// Unimock checks `next_call` clauses in order, so this clause follows any input read.
pub(crate) fn connects() -> impl Clause {
    ConnectMock.next_call(matching!(_)).answers(&|mock, _| {
        let resources: Arc<dyn ConnectResources + Send + Sync> = Arc::new(mock.clone());
        Ok(resources)
    })
}

/// Defines a clause that serves one resource handle from the same mock.
macro_rules! handle {
    ($(#[$doc:meta])* $name:ident, $method:ident, $api:ident) => {
        $(#[$doc])*
        pub(crate) fn $name() -> impl Clause {
            ConnectResourcesMock::$method
                .next_call(matching!())
                .answers(&|mock| {
                    let api: Arc<dyn $api + Send + Sync> = Arc::new(mock.clone());
                    api
                })
        }
    };
}

handle!(
    /// Serves the app handle from the same mock.
    apps_handle, apps, AppsApi
);
handle!(
    /// Serves the category handle from the same mock.
    categories_handle, app_categories, AppCategoriesApi
);
handle!(
    /// Serves the component handle from the same mock.
    components_handle, components, ComponentsApi
);
handle!(
    /// Serves the action handle from the same mock.
    actions_handle, actions, ActionsApi
);
handle!(
    /// Serves the token handle from the same mock.
    tokens_handle, tokens, TokensApi
);
handle!(
    /// Serves the account handle from the same mock.
    accounts_handle, accounts, AccountsApi
);
handle!(
    /// Serves the user handle from the same mock.
    users_handle, users, UsersApi
);
handle!(
    /// Serves the proxy handle from the same mock.
    proxy_handle, proxy, ProxyApi
);
handle!(
    /// Serves the deployed trigger handle from the same mock.
    deployed_triggers_handle, deployed_triggers, DeployedTriggersApi
);
handle!(
    /// Serves the registry trigger handle from the same mock.
    triggers_handle, triggers, TriggersApi
);
handle!(
    /// Serves the project handle from the same mock.
    projects_handle, projects, ProjectsApi
);
handle!(
    /// Serves the usage handle from the same mock.
    usage_handle, usage, UsageApi
);
handle!(
    /// Serves the project webhook handle from the same mock.
    webhooks_handle, project_webhooks, ProjectWebhooksApi
);

/// Returns the exact standard output of a command that prints `value`.
pub(crate) fn printed<T: Serialize>(value: &T) -> String {
    format!("{}\n", serde_json::to_string_pretty(value).unwrap())
}
