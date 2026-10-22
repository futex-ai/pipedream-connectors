#![doc = include_str!("../README.md")]
#![warn(unreachable_pub)]

#[cfg(feature = "reqwest")]
pub mod builder;
pub mod client;
pub mod config;
pub mod error;
mod error_debug;
mod failure;
mod headers;
mod json;
pub mod pagination;
pub mod request;
#[cfg(feature = "reqwest")]
pub mod reqwest_transport;
pub mod resources;
pub mod retry;
pub mod runtime;
pub mod token;
mod token_failure;
pub mod transport;
pub mod validation;

pub use client::PipedreamClient;
pub use resources::ConnectResources;
#[cfg(any(test, feature = "unimock"))]
pub use resources::ConnectResourcesMock;

#[cfg(test)]
#[path = "_tests_/mod.rs"]
mod tests;
