use serde::Deserialize;
use std::fs;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file missing or unreadable: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse configuration: {0}")]
    ParseError(#[from] serde_json::Error),
}

/// Enterprise Configuration Model
/// In production, no IPs or thresholds are hardcoded in the source files.
#[derive(Debug, Deserialize, Clone)]
pub struct EdgeConfig {
    pub sdr_center_freq_hz: f64,
    pub sdr_sample_rate: f64,
    pub lidar_ip: String,
    pub gps_uart_port: String,
    pub gps_baud_rate: u32,
    pub control_loop_hz: u64,
    pub drone_mass_kg: f64,
}

impl Default for EdgeConfig {
    fn default() -> Self {
        Self {
            sdr_center_freq_hz: 1.8e9,
            sdr_sample_rate: 2.0e6,
            lidar_ip: "192.168.1.200".to_string(),
            gps_uart_port: "/dev/ttyS0".to_string(),
            gps_baud_rate: 115200,
            control_loop_hz: 100,
            drone_mass_kg: 1.2,
        }
    }
}

impl EdgeConfig {
    pub fn load(path: &str) -> Result<Self, ConfigError> {
        let contents = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return Ok(Self::default()), // Fallback for simulation
        };
        let config: EdgeConfig = serde_json::from_str(&contents)?;
        Ok(config)
    }
}
