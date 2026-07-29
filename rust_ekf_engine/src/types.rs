use nalgebra::{Matrix6, Vector3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentVector {
    pub target_heading_rad: f64,
    pub target_climb_rate_mps: f64,
    pub target_speed_mps: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NavigationTier {
    Tier1GnssActive = 1,
    Tier2CellularRssi = 2,
    Tier3VisualSlam = 3,
    Tier4EmergencyLanding = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EKFState {
    pub position: [f64; 3], // x, y, z in ENU
    pub velocity: [f64; 3], // vx, vy, vz

    #[serde(skip)] // Complex matrix not usually needed over wire, but can be customized
    pub covariance: Matrix6<f64>,

    pub current_tier: NavigationTier,
    pub jnr_db: f64,
    pub handover_alpha: f64,
    pub estimated_rms_error_m: f64,
    pub handover_in_progress: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    pub drone_id: String,
    pub state: EKFState,
    pub timestamp: f64,
}
