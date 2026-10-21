//! Typed resource handles that share authentication, transport, and error mapping.

mod accessors;
pub mod actions;
pub mod app_categories;
pub mod apps;
mod catalog;
pub mod components;
mod execution;
mod queries;
pub mod triggers;

#[cfg(test)]
#[path = "_tests_/support.rs"]
mod test_support;
