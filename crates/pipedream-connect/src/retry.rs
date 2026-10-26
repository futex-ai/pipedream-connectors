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

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: NonZeroU32::new(3).map_or(NonZeroU32::MIN, |value| value),
            max_delay: Duration::from_secs(30),
            fallback_delay: Duration::from_secs(1),
        }
    }
}

impl RetryPolicy {
    /// Selects total attempts, maximum delay, and delay without Retry-After.
    ///
    /// All values are explicit. Fallback delays are clamped to `max_delay`.
    pub fn new(max_attempts: NonZeroU32, max_delay: Duration, fallback_delay: Duration) -> Self {
        Self {
            max_attempts,
            max_delay,
            fallback_delay,
        }
    }

    /// Returns no delay when Retry-After exceeds the cap, which stops retries.
    pub fn delay(&self, retry_after: Option<Duration>) -> Option<Duration> {
        match retry_after {
            Some(delay) if delay > self.max_delay => None,
            Some(delay) => Some(delay),
            None => Some(self.fallback_delay.min(self.max_delay)),
        }
    }

    pub(crate) fn retries(&self, status: u16, attempts: u32) -> bool {
        attempts < self.max_attempts.get() && matches!(status, 408 | 429 | 500..=599)
    }
}
