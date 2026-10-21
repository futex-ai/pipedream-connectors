//! Tests for flag precedence, variable fallbacks, and setting validation.

use std::ffi::OsString;
#[cfg(unix)]
use std::os::unix::ffi::OsStringExt;

use pipedream_connect_types::{Environment, SecretString};
use unimock::{MockFn, Unimock, matching};

use crate::cli::{EnvironmentArg, GlobalArgs};
use crate::error::Error;
use crate::settings::{ApiOrigin, Setting, Supply, resolve};
use crate::test_support::{Env, SECRET};
use crate::variables::GetVariableMock;

fn no_flags() -> GlobalArgs {
    GlobalArgs {
        client_id: None,
        client_secret: None,
        project_id: None,
        environment: None,
        api_origin: None,
        show_secrets: false,
    }
}

fn variables(env: Env) -> Unimock {
    Unimock::new(env.clauses())
}

#[test]
fn variables_supply_every_setting() {
    let settings = resolve(&no_flags(), &variables(Env::CONFIGURED)).unwrap();
    assert_eq!(settings.project_id.as_str(), "proj_Example123");
    assert_eq!(settings.environment, Environment::Development);
    assert_eq!(settings.client_id.expose_secret(), "client-id-value");
    assert_eq!(settings.client_secret.expose_secret(), SECRET);
    assert_eq!(settings.api_origin, None);
    assert_eq!(settings.origin(), "https://api.pipedream.com");
    assert_eq!(settings.custom_origin(), None);
    let config = settings.client_config().unwrap();
    assert_eq!(config.project_id().as_str(), "proj_Example123");
    assert_eq!(config.environment(), &Environment::Development);
}

/// Unimock fails any variable read here, so the flags must supply every setting.
#[test]
fn flags_take_precedence_over_variables() {
    let flags = GlobalArgs {
        client_id: Some("flag-id".to_owned()),
        client_secret: Some(SecretString::new("flag-secret")),
        project_id: Some("proj_Flag1".to_owned()),
        environment: Some(EnvironmentArg::Production),
        api_origin: Some("https://api.example.com".to_owned()),
        show_secrets: false,
    };
    let settings = resolve(&flags, &Unimock::new(())).unwrap();
    assert_eq!(settings.client_id.expose_secret(), "flag-id");
    assert_eq!(settings.client_secret.expose_secret(), "flag-secret");
    assert_eq!(settings.project_id.as_str(), "proj_Flag1");
    assert_eq!(settings.environment, Environment::Production);
    assert_eq!(settings.origin(), "https://api.example.com");
    assert_eq!(
        settings.custom_origin(),
        Some(&ApiOrigin {
            url: "https://api.example.com".to_owned(),
            supply: Supply::Flag,
        })
    );
}

#[test]
fn origin_variable_applies_when_the_flag_is_absent() {
    let env = Env {
        api_origin: Some("https://variable.example.com"),
        ..Env::CONFIGURED
    };
    let settings = resolve(&no_flags(), &variables(env)).unwrap();
    assert_eq!(settings.origin(), "https://variable.example.com");
    assert_eq!(
        settings.custom_origin().map(|origin| origin.supply),
        Some(Supply::Variable)
    );
}

#[test]
fn the_default_origin_is_not_custom_even_when_it_is_set() {
    let env = Env {
        api_origin: Some("https://api.pipedream.com"),
        ..Env::CONFIGURED
    };
    let settings = resolve(&no_flags(), &variables(env)).unwrap();
    assert_eq!(settings.origin(), "https://api.pipedream.com");
    assert_eq!(settings.custom_origin(), None);
}

#[test]
fn every_missing_setting_is_reported_at_once() {
    let error = resolve(&no_flags(), &variables(Env::EMPTY)).unwrap_err();
    assert!(matches!(
        error,
        Error::MissingSettings { settings } if settings == [
            Setting::ClientId,
            Setting::ClientSecret,
            Setting::ProjectId,
            Setting::Environment,
        ]
    ));
    for (env, missing) in [
        (
            Env {
                client_secret: None,
                ..Env::CONFIGURED
            },
            Setting::ClientSecret,
        ),
        (
            Env {
                project_id: None,
                ..Env::CONFIGURED
            },
            Setting::ProjectId,
        ),
        (
            Env {
                environment: None,
                ..Env::CONFIGURED
            },
            Setting::Environment,
        ),
    ] {
        let error = resolve(&no_flags(), &variables(env)).unwrap_err();
        assert!(
            matches!(&error, Error::MissingSettings { settings } if settings == &[missing]),
            "{missing:?}: {error:?}"
        );
    }
}

#[test]
fn empty_variables_count_as_unset() {
    let env = Env {
        client_id: Some(""),
        ..Env::CONFIGURED
    };
    let error = resolve(&no_flags(), &variables(env)).unwrap_err();
    assert!(matches!(
        error,
        Error::MissingSettings { settings } if settings == [Setting::ClientId]
    ));
}

#[test]
fn invalid_project_id_names_its_source() {
    let env = Env {
        project_id: Some("project-1"),
        ..Env::CONFIGURED
    };
    let error = resolve(&no_flags(), &variables(env)).unwrap_err();
    assert!(matches!(
        error,
        Error::InvalidSetting {
            setting: Setting::ProjectId,
            supply: Supply::Variable
        }
    ));
    let flags = GlobalArgs {
        project_id: Some("project-1".to_owned()),
        ..no_flags()
    };
    let error = resolve(&flags, &variables(Env::CONFIGURED)).unwrap_err();
    assert!(matches!(
        error,
        Error::InvalidSetting {
            setting: Setting::ProjectId,
            supply: Supply::Flag
        }
    ));
}

#[test]
fn environment_variable_accepts_only_the_two_environments() {
    let env = Env {
        environment: Some("Production"),
        ..Env::CONFIGURED
    };
    let error = resolve(&no_flags(), &variables(env)).unwrap_err();
    assert!(matches!(
        error,
        Error::InvalidSetting {
            setting: Setting::Environment,
            supply: Supply::Variable
        }
    ));
    let env = Env {
        environment: Some("production"),
        ..Env::CONFIGURED
    };
    let settings = resolve(&no_flags(), &variables(env)).unwrap();
    assert_eq!(settings.environment, Environment::Production);
}

#[test]
fn invalid_credentials_name_both_sources() {
    let flags = GlobalArgs {
        client_id: Some(" padded-id".to_owned()),
        ..no_flags()
    };
    let error = resolve(&flags, &variables(Env::CONFIGURED)).unwrap_err();
    assert!(matches!(
        error,
        Error::InvalidCredentials {
            client_id: Supply::Flag,
            client_secret: Supply::Variable
        }
    ));
}

#[test]
fn invalid_origin_names_its_source() {
    let env = Env {
        api_origin: Some("http://api.example.com"),
        ..Env::CONFIGURED
    };
    let error = resolve(&no_flags(), &variables(env)).unwrap_err();
    assert!(matches!(
        error,
        Error::InvalidSetting {
            setting: Setting::ApiOrigin,
            supply: Supply::Variable
        }
    ));
    let flags = GlobalArgs {
        api_origin: Some("https://api.example.com/v1".to_owned()),
        ..no_flags()
    };
    let error = resolve(&flags, &variables(Env::CONFIGURED)).unwrap_err();
    assert!(matches!(
        error,
        Error::InvalidSetting {
            setting: Setting::ApiOrigin,
            supply: Supply::Flag
        }
    ));
}

#[cfg(unix)]
#[test]
fn variable_that_is_not_utf8_is_unreadable() {
    let variables = Unimock::new((
        GetVariableMock
            .each_call(matching!(Setting::ClientId))
            .returns(Some(OsString::from_vec(vec![0xff, 0xfe]))),
        Env::CONFIGURED.clauses(),
    ));
    let error = resolve(&no_flags(), &variables).unwrap_err();
    assert!(matches!(
        error,
        Error::UnreadableVariable {
            setting: Setting::ClientId
        }
    ));
}

#[test]
fn setting_names_match_the_documented_flags_and_variables() {
    assert_eq!(Setting::ClientSecret.flag(), "--client-secret");
    assert_eq!(Setting::ClientSecret.variable(), "PIPEDREAM_CLIENT_SECRET");
    assert_eq!(Supply::Flag.name(Setting::ApiOrigin), "--api-origin");
    assert_eq!(
        Supply::Variable.name(Setting::ApiOrigin),
        "PIPEDREAM_API_ORIGIN"
    );
    assert_eq!(
        Setting::ProjectId.to_string(),
        "--project-id or PIPEDREAM_PROJECT_ID"
    );
    assert_eq!(Supply::Variable.to_string(), "variable");
}
