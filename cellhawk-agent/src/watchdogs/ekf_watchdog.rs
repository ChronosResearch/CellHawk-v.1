use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq)]
pub enum WatchdogStatus {
    Healthy,
    Expired,
}

/// Monitors the EKF update rate. If the EKF stops updating
/// for > 500ms, trigger a fail-safe.
pub struct EkfWatchdog {
    last_update: Instant,
    threshold: Duration,
}

impl EkfWatchdog {
    pub fn new(threshold_ms: u64) -> Self {
        Self {
            last_update: Instant::now(),
            threshold: Duration::from_millis(threshold_ms),
        }
    }
    
    pub fn pet(&mut self) {
        self.last_update = Instant::now();
    }
    
    pub fn check(&self) -> WatchdogStatus {
        if self.last_update.elapsed() > self.threshold {
            WatchdogStatus::Expired
        } else {
            WatchdogStatus::Healthy
        }
    }
}
