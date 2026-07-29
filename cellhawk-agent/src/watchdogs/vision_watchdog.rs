use std::time::{Duration, Instant};
use crate::watchdogs::ekf_watchdog::WatchdogStatus;

/// Monitors the SLAM pose updates. If no pose is returned
/// for > 300ms, trigger a fail-safe (fallback to IMU-only).
pub struct VisionWatchdog {
    last_pose: Instant,
    threshold: Duration,
}

impl VisionWatchdog {
    pub fn new(threshold_ms: u64) -> Self {
        Self {
            last_pose: Instant::now(),
            threshold: Duration::from_millis(threshold_ms),
        }
    }
    
    pub fn pet(&mut self) {
        self.last_pose = Instant::now();
    }
    
    pub fn check(&self) -> WatchdogStatus {
        if self.last_pose.elapsed() > self.threshold {
            WatchdogStatus::Expired
        } else {
            WatchdogStatus::Healthy
        }
    }
}
