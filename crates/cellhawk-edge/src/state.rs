use nalgebra::Vector3;

#[derive(Debug, Clone)]
pub struct StateEstimate {
    pub position: Vector3<f64>,
    pub velocity: Vector3<f64>,
    pub attitude_euler: Vector3<f64>, // Roll, Pitch, Yaw
}

impl Default for StateEstimate {
    fn default() -> Self {
        Self {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
            attitude_euler: Vector3::zeros(),
        }
    }
}
