//! Typed resource handles that share authentication, transport, and error mapping.

mod accessors;
pub use accessors::ConnectResources;
#[cfg(any(test, feature = "unimock"))]
pub use accessors::ConnectResourcesMock;
pub mod accounts;
pub mod actions;
mod admin_operations;
pub mod app_categories;
pub mod app_overrides;
pub mod apps;
mod auth_operations;
pub mod components;
pub mod deployed_triggers;
pub mod file_stash;
pub mod oauth_apps;
mod operation;
pub mod project_webhooks;
pub mod projects;
pub mod proxy;
pub mod rate_limits;
pub mod tokens;
mod trigger_operations;
pub mod triggers;
pub mod usage;
pub mod users;
pub mod workflows;

#[cfg(test)]
#[path = "_tests_/support.rs"]
mod test_support;

#[cfg(test)]
#[path = "_tests_/accounts_tests.rs"]
mod accounts_tests;

#[cfg(test)]
#[path = "_tests_/proxy_tests.rs"]
mod proxy_tests;

#[cfg(test)]
#[path = "_tests_/tokens_tests.rs"]
mod tokens_tests;

#[cfg(test)]
#[path = "_tests_/users_tests.rs"]
mod users_tests;
