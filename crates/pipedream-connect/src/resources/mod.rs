//! Typed resource handles that share authentication, transport, and error mapping.

mod accessors;
pub use accessors::ConnectResources;
#[cfg(any(test, feature = "unimock"))]
pub use accessors::ConnectResourcesMock;
pub mod accounts;
pub mod actions;
pub mod app_categories;
pub mod apps;
mod auth_operations;
pub mod components;
mod operation;
pub mod proxy;
pub mod tokens;
pub mod triggers;
pub mod users;

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
