//! Async authentication and dispatch for the Pipedream Connect API.

#![warn(unreachable_pub)]

#[cfg(feature = "reqwest")]
pub mod builder;
pub mod client;
pub mod config;
pub mod error;
mod failure;
mod headers;
mod json;
pub mod pagination;
pub mod request;
#[cfg(feature = "reqwest")]
pub mod reqwest_transport;
pub mod retry;
pub mod runtime;
pub mod token;
pub mod transport;

pub use client::PipedreamClient;

#[cfg(test)]
#[path = "_tests_/mod.rs"]
mod tests;
