use std::time::{Duration, Instant};
use crate::watchdogs::ekf_watchdog::WatchdogStatus;

/// Monitors Redis connectivity. If Redis is unreachable
/// for > 5 seconds, trigger a fail-safe (operate in standalone mode).
pub struct SwarmWatchdog {
    last_ping: Instant,
    threshold: Duration,
}

impl SwarmWatchdog {
    pub fn new(threshold_ms: u64) -> Self {
        Self {
            last_ping: Instant::now(),
            threshold: Duration::from_millis(threshold_ms),
        }
    }
    
    pub fn pet(&mut self) {
        self.last_ping = Instant::now();
    }
    
    pub fn check(&self) -> WatchdogStatus {
        if self.last_ping.elapsed() > self.threshold {
            WatchdogStatus::Expired
        } else {
            WatchdogStatus::Healthy
        }
    }
}
