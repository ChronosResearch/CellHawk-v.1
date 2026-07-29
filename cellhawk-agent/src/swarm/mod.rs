pub mod publisher;
pub mod subscriber;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hazard {
    pub lat: f64,
    pub lon: f64,
    pub level: u8,
    pub ts: u64,
}
