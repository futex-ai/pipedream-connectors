//! Typed resource handles that share authentication, transport, and error mapping.

mod accessors;
pub use accessors::ConnectResources;
#[cfg(any(test, feature = "unimock"))]
pub use accessors::ConnectResourcesMock;
pub mod actions;
pub mod app_categories;
pub mod apps;
pub mod components;
mod operation;
pub mod triggers;

#[cfg(test)]
#[path = "_tests_/support.rs"]
mod test_support;
