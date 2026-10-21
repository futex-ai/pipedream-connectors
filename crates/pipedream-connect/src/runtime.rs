//! Injectable time and sleep boundaries.

use std::time::{Duration, SystemTime};

use async_trait::async_trait;

/// Supplies wall time for token expiry and HTTP dates.
#[cfg_attr(any(test, feature = "unimock"), unimock::unimock(api = ClockMock))]
pub trait Clock: Send + Sync {
    /// Returns the current wall time.
    fn now(&self) -> SystemTime;
}

/// System wall clock for a composition root.
#[derive(Debug)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
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
