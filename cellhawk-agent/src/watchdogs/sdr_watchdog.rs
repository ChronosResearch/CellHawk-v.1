use std::time::{Duration, Instant};
use crate::watchdogs::ekf_watchdog::WatchdogStatus;

/// Monitors the SDR RSSI data stream. If no RSSI samples are
/// received for > 200ms, trigger a fail-safe (fallback to visual only).
pub struct SdrWatchdog {
    last_sample: Instant,
    threshold: Duration,
}

impl SdrWatchdog {
    pub fn new(threshold_ms: u64) -> Self {
        Self {
            last_sample: Instant::now(),
            threshold: Duration::from_millis(threshold_ms),
        }
    }
    
    pub fn pet(&mut self) {
        self.last_sample = Instant::now();
    }
    
    pub fn check(&self) -> WatchdogStatus {
        if self.last_sample.elapsed() > self.threshold {
            WatchdogStatus::Expired
        } else {
            WatchdogStatus::Healthy
        }
    }
}
