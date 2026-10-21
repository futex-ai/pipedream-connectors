//! Connection settings from the global flags, with environment-variable fallbacks.

use std::fmt;

use clap::ValueEnum;
use pipedream_connect::config::{Authentication, ClientConfig, ClientCredentials};
use pipedream_connect::error::InvalidRequest;
use pipedream_connect_types::{Environment, ProjectId, SecretString};

use crate::cli::{EnvironmentArg, GlobalArgs};
use crate::error::{Error, Result};
use crate::variables::Variables;

/// The API origin that the SDK uses when the caller selects none.
pub(crate) const DEFAULT_ORIGIN: &str = "https://api.pipedream.com";

/// A connection setting that a flag or an environment variable supplies.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Setting {
    /// The OAuth client ID.
    ClientId,
    /// The OAuth client secret.
    ClientSecret,
    /// The Connect project ID.
    ProjectId,
    /// The project environment.
    Environment,
    /// The API origin.
    ApiOrigin,
}

impl Setting {
    /// Returns the command-line flag of this setting.
    pub(crate) fn flag(self) -> &'static str {
        match self {
            Self::ClientId => "--client-id",
            Self::ClientSecret => "--client-secret",
            Self::ProjectId => "--project-id",
            Self::Environment => "--environment",
            Self::ApiOrigin => "--api-origin",
        }
    }

    /// Returns the environment variable of this setting.
    pub(crate) fn variable(self) -> &'static str {
        match self {
            Self::ClientId => "PIPEDREAM_CLIENT_ID",
            Self::ClientSecret => "PIPEDREAM_CLIENT_SECRET",
            Self::ProjectId => "PIPEDREAM_PROJECT_ID",
            Self::Environment => "PIPEDREAM_ENVIRONMENT",
            Self::ApiOrigin => "PIPEDREAM_API_ORIGIN",
        }
    }
}

impl fmt::Display for Setting {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} or {}", self.flag(), self.variable())
    }
}

/// How the caller supplied a setting value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Supply {
    /// A command-line flag.
    Flag,
    /// An environment variable.
    Variable,
}

impl fmt::Display for Supply {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Flag => "flag",
            Self::Variable => "variable",
        })
    }
}

impl Supply {
    /// Returns the flag or the variable that supplied `setting`.
    pub(crate) fn name(self, setting: Setting) -> &'static str {
        match self {
            Self::Flag => setting.flag(),
            Self::Variable => setting.variable(),
        }
    }
}

/// An API origin and the way the caller supplied it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ApiOrigin {
    /// The origin URL.
    pub(crate) url: String,
    /// The flag or variable that supplied it.
    pub(crate) supply: Supply,
}

/// Validated connection settings for one command.
#[derive(Clone, Debug)]
pub(crate) struct Settings {
    /// The Connect project.
    pub(crate) project_id: ProjectId,
    /// The project environment.
    pub(crate) environment: Environment,
    /// The OAuth client ID.
    pub(crate) client_id: SecretString,
    /// The OAuth client secret.
    pub(crate) client_secret: SecretString,
    /// The API origin, when the caller selected one.
    pub(crate) api_origin: Option<ApiOrigin>,
}

impl Settings {
    /// Returns the API origin that requests use.
    pub(crate) fn origin(&self) -> &str {
        match &self.api_origin {
            Some(origin) => &origin.url,
            None => DEFAULT_ORIGIN,
        }
    }

    /// Returns the origin when the caller selected one other than the default.
    pub(crate) fn custom_origin(&self) -> Option<&ApiOrigin> {
        self.api_origin
            .as_ref()
            .filter(|origin| origin.url != DEFAULT_ORIGIN)
    }

    /// Builds the SDK client configuration. Sends no request.
    pub(crate) fn client_config(&self) -> pipedream_connect::error::Result<ClientConfig> {
        let credentials = ClientCredentials::new(
            self.client_id.expose_secret(),
            self.client_secret.expose_secret(),
        )?;
        let config = ClientConfig::new(
            self.project_id.clone(),
            self.environment.clone(),
            Authentication::ClientCredentials(credentials),
        )?;
        match &self.api_origin {
            Some(origin) => config.with_origin(&origin.url),
            None => Ok(config),
        }
    }
}

/// A raw setting value and the way the caller supplied it.
struct Supplied {
    value: String,
    supply: Supply,
}

/// Resolves each setting from its flag, then from its variable, and validates the result.
pub(crate) fn resolve(args: &GlobalArgs, variables: &dyn Variables) -> Result<Settings> {
    let client_id = lookup(Setting::ClientId, args.client_id.clone(), variables)?;
    let secret_flag = args
        .client_secret
        .as_ref()
        .map(|secret| secret.expose_secret().to_owned());
    let client_secret = lookup(Setting::ClientSecret, secret_flag, variables)?;
    let project = lookup(Setting::ProjectId, args.project_id.clone(), variables)?;
    let environment = environment(args.environment, variables)?;
    let origin = lookup(Setting::ApiOrigin, args.api_origin.clone(), variables)?;
    let missing: Vec<Setting> = [
        (Setting::ClientId, client_id.is_none()),
        (Setting::ClientSecret, client_secret.is_none()),
        (Setting::ProjectId, project.is_none()),
        (Setting::Environment, environment.is_none()),
    ]
    .into_iter()
    .filter_map(|(setting, absent)| absent.then_some(setting))
    .collect();
    let (Some(client_id), Some(client_secret), Some(project), Some(environment)) =
        (client_id, client_secret, project, environment)
    else {
        return Err(Error::MissingSettings { settings: missing });
    };
    let project_id = match ProjectId::new(project.value) {
        Ok(project_id) => project_id,
        Err(_) => return Err(invalid(Setting::ProjectId, project.supply)),
    };
    let origin_supply = origin.as_ref().map(|origin| origin.supply);
    let settings = Settings {
        project_id,
        environment,
        client_id: SecretString::new(client_id.value),
        client_secret: SecretString::new(client_secret.value),
        api_origin: origin.map(|origin| ApiOrigin {
            url: origin.value,
            supply: origin.supply,
        }),
    };
    let cause = match settings.client_config() {
        Ok(_) => return Ok(settings),
        Err(cause) => cause,
    };
    Err(match (cause, origin_supply) {
        (pipedream_connect::error::Error::InvalidRequest(InvalidRequest::Credentials), _) => {
            Error::InvalidCredentials {
                client_id: client_id.supply,
                client_secret: client_secret.supply,
            }
        }
        (pipedream_connect::error::Error::InvalidRequest(InvalidRequest::Origin), Some(supply)) => {
            invalid(Setting::ApiOrigin, supply)
        }
        (cause, _) => Error::Client(cause),
    })
}

/// Reads the environment from its flag, then from its variable, with one grammar.
fn environment(
    flag: Option<EnvironmentArg>,
    variables: &dyn Variables,
) -> Result<Option<Environment>> {
    if let Some(environment) = flag {
        return Ok(Some(environment.model()));
    }
    let Some(supplied) = lookup(Setting::Environment, None, variables)? else {
        return Ok(None);
    };
    match EnvironmentArg::from_str(&supplied.value, false) {
        Ok(environment) => Ok(Some(environment.model())),
        Err(_) => Err(invalid(Setting::Environment, supplied.supply)),
    }
}

/// Prefers the flag. An empty variable counts as unset.
fn lookup(
    setting: Setting,
    flag: Option<String>,
    variables: &dyn Variables,
) -> Result<Option<Supplied>> {
    if let Some(value) = flag {
        return Ok(Some(Supplied {
            value,
            supply: Supply::Flag,
        }));
    }
    let Some(raw) = variables.get(setting) else {
        return Ok(None);
    };
    if raw.is_empty() {
        return Ok(None);
    }
    match raw.into_string() {
        Ok(value) => Ok(Some(Supplied {
            value,
            supply: Supply::Variable,
        })),
        Err(_) => Err(Error::UnreadableVariable { setting }),
    }
}

fn invalid(setting: Setting, supply: Supply) -> Error {
    Error::InvalidSetting { setting, supply }
}

#[cfg(test)]
#[path = "_tests_/settings_tests.rs"]
mod settings_tests;
