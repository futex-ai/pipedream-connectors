//! Explicit GET retry budgets without SDK retry defaults.

use std::num::NonZeroU32;
use std::time::Duration;

/// A caller-selected GET attempt and delay budget.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetryPolicy {
    pub(crate) max_attempts: NonZeroU32,
    max_delay: Duration,
    fallback_delay: Duration,
}

impl RetryPolicy {
    /// Selects total attempts, maximum delay, and delay without Retry-After.
    ///
    /// All values are explicit. Each delay is clamped to `max_delay`.
    pub fn new(max_attempts: NonZeroU32, max_delay: Duration, fallback_delay: Duration) -> Self {
        Self {
            max_attempts,
            max_delay,
            fallback_delay,
        }
    }

    /// Returns the delay for the next attempt, within the configured cap.
    pub fn delay(&self, retry_after: Option<Duration>) -> Duration {
        retry_after
            .unwrap_or(self.fallback_delay)
            .min(self.max_delay)
    }

    pub(crate) fn retries(&self, status: u16, attempts: u32) -> bool {
        attempts < self.max_attempts.get() && matches!(status, 408 | 429 | 500..=599)
    }
}
