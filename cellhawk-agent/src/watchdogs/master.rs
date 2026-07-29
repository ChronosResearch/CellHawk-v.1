use crate::watchdogs::ekf_watchdog::{EkfWatchdog, WatchdogStatus};
use crate::watchdogs::sdr_watchdog::SdrWatchdog;
use crate::watchdogs::vision_watchdog::VisionWatchdog;
use crate::watchdogs::swarm_watchdog::SwarmWatchdog;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchdogEvent {
    EkfTimeout,
    SdrTimeout,
    VisionTimeout,
    SwarmTimeout,
}

/// Aggregates all watchdogs and triggers unified fail-safes.
pub struct MasterWatchdog {
    pub ekf: EkfWatchdog,
    pub sdr: SdrWatchdog,
    pub vision: VisionWatchdog,
    pub swarm: SwarmWatchdog,
}

impl MasterWatchdog {
    pub fn new() -> Self {
        Self {
            ekf: EkfWatchdog::new(500),
            sdr: SdrWatchdog::new(200),
            vision: VisionWatchdog::new(300),
            swarm: SwarmWatchdog::new(5000),
        }
    }

    pub fn check_all(&mut self) -> Vec<WatchdogEvent> {
        let mut events = Vec::new();
        
        if self.ekf.check() == WatchdogStatus::Expired {
            events.push(WatchdogEvent::EkfTimeout);
        }
        
        if self.sdr.check() == WatchdogStatus::Expired {
            events.push(WatchdogEvent::SdrTimeout);
        }
        
        if self.vision.check() == WatchdogStatus::Expired {
            events.push(WatchdogEvent::VisionTimeout);
        }
        
        if self.swarm.check() == WatchdogStatus::Expired {
            events.push(WatchdogEvent::SwarmTimeout);
        }
        
        events
    }
}
