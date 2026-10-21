//! Typed, lossless data models for the Pipedream Connect API.

#![warn(unreachable_pub)]

mod accounts;
mod admin;
mod error;
mod execution;
mod ids;
mod pagination;
mod props;
mod registry;
mod secret;
mod shared;
mod triggers;
mod wire;

pub use accounts::*;
pub use admin::*;
pub use error::*;
pub use execution::*;
pub use ids::*;
pub use pagination::*;
pub use props::*;
pub use registry::*;
pub use secret::*;
pub use shared::*;
pub use triggers::*;
pub use wire::*;

#[cfg(test)]
#[path = "_tests_/models_tests.rs"]
mod models_tests;
