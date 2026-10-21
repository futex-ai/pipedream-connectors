//! Injectable time and sleep boundaries.

use std::time::{Duration, Instant, SystemTime};

use async_trait::async_trait;

/// Supplies monotonic time for token expiry and wall time for HTTP dates.
#[cfg_attr(any(test, feature = "unimock"), unimock::unimock(api = ClockMock))]
pub trait Clock: Send + Sync {
    /// Returns the current wall time.
    fn now(&self) -> SystemTime;
    /// Returns monotonic time since an origin owned by this clock.
    fn monotonic(&self) -> Duration;
}

/// System clock with its own monotonic origin for a composition root.
#[derive(Debug)]
pub struct SystemClock {
    origin: Instant,
}

impl Default for SystemClock {
    fn default() -> Self {
        Self {
            origin: Instant::now(),
        }
    }
}

impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }

    fn monotonic(&self) -> Duration {
        self.origin.elapsed()
    }
}

/// Waits between explicitly enabled GET attempts.
#[cfg_attr(any(test, feature = "unimock"), unimock::unimock(api = SleeperMock))]
#[async_trait]
pub trait Sleeper: Send + Sync {
    /// Waits for a caller-selected duration.
    async fn sleep(&self, duration: Duration);
}

/// Tokio sleeper for runtime composition.
#[derive(Debug)]
pub struct TokioSleeper;

#[async_trait]
impl Sleeper for TokioSleeper {
    async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }
}
