//! Number flags that the CLI reads as text, so that each failure names its flag.

use std::num::NonZeroUsize;
use std::str::FromStr;

use crate::argument::Argument;
use crate::error::{Error, Result};

/// Parses a number that an SDK rule bounds. Text that is not such a number becomes zero.
///
/// Every bounded SDK value rejects zero, so the SDK rule then reports its documented range.
pub(crate) fn bounded<T: FromStr + Default>(text: &str) -> T {
    text.parse().unwrap_or_default()
}

/// Parses a count of 1 or more without an upper limit.
pub(crate) fn positive(text: &str, argument: Argument) -> Result<NonZeroUsize> {
    match text.parse::<NonZeroUsize>() {
        Ok(count) => Ok(count),
        Err(_) => Err(Error::InvalidCount { argument }),
    }
}
