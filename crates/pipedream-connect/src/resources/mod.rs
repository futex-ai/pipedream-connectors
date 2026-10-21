//! Typed resource handles that share authentication, transport, and error mapping.

mod accessors;
pub use accessors::ConnectResources;
#[cfg(any(test, feature = "unimock"))]
pub use accessors::ConnectResourcesMock;
pub mod actions;
mod admin_operations;
pub mod app_categories;
pub mod app_overrides;
pub mod apps;
pub mod components;
pub mod file_stash;
pub mod oauth_apps;
mod operation;
pub mod project_webhooks;
pub mod projects;
pub mod rate_limits;
pub mod triggers;
pub mod usage;
pub mod workflows;

#[cfg(test)]
#[path = "_tests_/support.rs"]
mod test_support;
