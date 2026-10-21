//! A harness that runs the CLI against one mock for every process boundary.

use std::ffi::OsString;
use std::sync::Arc;

use pipedream_connect::resources::ConnectResources;
use pipedream_connect::resources::ConnectResourcesMock;
use pipedream_connect::resources::actions::ActionsApi;
use pipedream_connect::resources::app_categories::AppCategoriesApi;
use pipedream_connect::resources::apps::AppsApi;
use pipedream_connect::resources::components::ComponentsApi;
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

/// Serves the app handle from the same mock.
pub(crate) fn apps_handle() -> impl Clause {
    ConnectResourcesMock::apps
        .next_call(matching!())
        .answers(&|mock| {
            let api: Arc<dyn AppsApi + Send + Sync> = Arc::new(mock.clone());
            api
        })
}

/// Serves the category handle from the same mock.
pub(crate) fn categories_handle() -> impl Clause {
    ConnectResourcesMock::app_categories
        .next_call(matching!())
        .answers(&|mock| {
            let api: Arc<dyn AppCategoriesApi + Send + Sync> = Arc::new(mock.clone());
            api
        })
}

/// Serves the component handle from the same mock.
pub(crate) fn components_handle() -> impl Clause {
    ConnectResourcesMock::components
        .next_call(matching!())
        .answers(&|mock| {
            let api: Arc<dyn ComponentsApi + Send + Sync> = Arc::new(mock.clone());
            api
        })
}

/// Serves the action handle from the same mock.
pub(crate) fn actions_handle() -> impl Clause {
    ConnectResourcesMock::actions
        .next_call(matching!())
        .answers(&|mock| {
            let api: Arc<dyn ActionsApi + Send + Sync> = Arc::new(mock.clone());
            api
        })
}

/// Returns the exact standard output of a command that prints `value`.
pub(crate) fn printed<T: Serialize>(value: &T) -> String {
    format!("{}\n", serde_json::to_string_pretty(value).unwrap())
}
