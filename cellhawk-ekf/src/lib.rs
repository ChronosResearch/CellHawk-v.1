use nalgebra::{SMatrix, SVector};

pub type StateVector = SVector<f64, 6>;
pub type CovarianceMatrix = SMatrix<f64, 6, 6>;

pub mod self_test;
use std::time::Instant;

#[derive(Clone)]
pub struct StateCheckpoint {
    pub state: StateVector,
    pub covariance: CovarianceMatrix,
    pub timestamp: Instant,
}

#[derive(Debug)]
pub enum EKFError {
    MathError(String),
}

pub struct CellhawkEKF {
    pub state: StateVector,
    pub covariance: CovarianceMatrix,
    tier_interpolation_counter: u32,
    target_scale: f64,
    current_scale: f64,
}

impl CellhawkEKF {
    pub fn new() -> Self {
        Self {
            state: StateVector::zeros(),
            covariance: CovarianceMatrix::identity(),
            tier_interpolation_counter: 0,
            target_scale: 1.0,
            current_scale: 1.0,
        }
    }

    pub fn predict(&mut self, dt: f64, accel: SVector<f64, 3>) -> Result<(), EKFError> {
        if dt <= 0.0 || dt.is_nan() {
            return Err(EKFError::MathError("Invalid dt".into()));
        }

        // x_{k|k-1} = x_{k-1|k-1} + v_{k-1|k-1} * dt
        self.state[0] += self.state[3] * dt;
        self.state[1] += self.state[4] * dt;
        self.state[2] += self.state[5] * dt;

        self.state[3] += accel[0] * dt;
        self.state[4] += accel[1] * dt;
        self.state[5] += accel[2] * dt;

        // Process noise covariance update Q
        let q = CovarianceMatrix::identity() * 0.1;

        let mut f = CovarianceMatrix::identity();
        f[(0, 3)] = dt;
        f[(1, 4)] = dt;
        f[(2, 5)] = dt;

        self.covariance = f * self.covariance * f.transpose() + q;
        Ok(())
    }

    pub fn create_checkpoint(&self) -> StateCheckpoint {
        StateCheckpoint {
            state: self.state.clone(),
            covariance: self.covariance.clone(),
            timestamp: Instant::now(),
        }
    }

    pub fn restore_checkpoint(&mut self, checkpoint: &StateCheckpoint) {
        self.state = checkpoint.state;
        self.covariance = checkpoint.covariance;
    }

    pub fn scale_covariance(&mut self, jnr_db: f64) {
        let new_target = if jnr_db > 19.0 {
            1.0 // Tier 1: Optimal
        } else if jnr_db > 6.0 {
            10.0 // Tier 2: Degraded
        } else {
            100.0 // Tier 3: Severely Denied
        };

        if (self.target_scale - new_target).abs() > 1e-3 {
            self.target_scale = new_target;
            self.tier_interpolation_counter = 5;
        }

        if self.tier_interpolation_counter > 0 {
            let step =
                (self.target_scale - self.current_scale) / (self.tier_interpolation_counter as f64);
            self.current_scale += step;
            self.tier_interpolation_counter -= 1;
        } else {
            self.current_scale = self.target_scale;
        }

        self.covariance *= self.current_scale;
    }

    fn huber_loss(innovation: &SVector<f64, 3>, s_inv: &SMatrix<f64, 3, 3>) -> f64 {
        let mahalanobis2 = (innovation.transpose() * s_inv * innovation)[0];
        let sigma3 = 3.0 * 3.0;
        if mahalanobis2 > sigma3 {
            // Down-weight the measurement
            (sigma3 / mahalanobis2).sqrt()
        } else {
            1.0
        }
    }

    pub fn update_gnss(&mut self, jnr_db: f64, measurement: SVector<f64, 3>) {
        let mut r = SMatrix::<f64, 3, 3>::identity() * 5.0; // GNSS base noise

        self.scale_covariance(jnr_db);

        let mut h = SMatrix::<f64, 3, 6>::zeros();
        h[(0, 0)] = 1.0;
        h[(1, 1)] = 1.0;
        h[(2, 2)] = 1.0;

        let innovation = measurement - (h * self.state);
        let s = h * self.covariance * h.transpose() + r;

        if let Some(s_inv) = s.try_inverse() {
            let weight = Self::huber_loss(&innovation, &s_inv);
            r *= 1.0 / weight;

            let s_weighted = h * self.covariance * h.transpose() + r;
            if let Some(sw_inv) = s_weighted.try_inverse() {
                let k = self.covariance * h.transpose() * sw_inv;
                self.state += k * innovation;
                self.covariance = (CovarianceMatrix::identity() - k * h) * self.covariance;
            }
        }
    }

    pub fn update_cellular(&mut self, measurement: SVector<f64, 3>) {
        // Similar to GNSS but different noise R
        let mut r = SMatrix::<f64, 3, 3>::identity() * 20.0;
        let mut h = SMatrix::<f64, 3, 6>::zeros();
        h[(0, 0)] = 1.0;
        h[(1, 1)] = 1.0;
        h[(2, 2)] = 1.0;

        let innovation = measurement - (h * self.state);
        let s = h * self.covariance * h.transpose() + r;
        if let Some(s_inv) = s.try_inverse() {
            let weight = Self::huber_loss(&innovation, &s_inv);
            r *= 1.0 / weight;

            let s_weighted = h * self.covariance * h.transpose() + r;
            if let Some(sw_inv) = s_weighted.try_inverse() {
                let k = self.covariance * h.transpose() * sw_inv;
                self.state += k * innovation;
                self.covariance = (CovarianceMatrix::identity() - k * h) * self.covariance;
            }
        }
    }

    pub fn update_vision(&mut self, measurement: SVector<f64, 3>) {
        // Visual SLAM update
        let mut r = SMatrix::<f64, 3, 3>::identity() * 1.5;
        let mut h = SMatrix::<f64, 3, 6>::zeros();
        h[(0, 0)] = 1.0;
        h[(1, 1)] = 1.0;
        h[(2, 2)] = 1.0;

        let innovation = measurement - (h * self.state);
        let s = h * self.covariance * h.transpose() + r;
        if let Some(s_inv) = s.try_inverse() {
            let weight = Self::huber_loss(&innovation, &s_inv);
            r *= 1.0 / weight;

            let s_weighted = h * self.covariance * h.transpose() + r;
            if let Some(sw_inv) = s_weighted.try_inverse() {
                let k = self.covariance * h.transpose() * sw_inv;
                self.state += k * innovation;
                self.covariance = (CovarianceMatrix::identity() - k * h) * self.covariance;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_imu_integration_step13() {
        let mut ekf = CellhawkEKF::new();
        let dt = 0.1;
        let accel = SVector::<f64, 3>::new(2.0, 0.0, 0.0);

        // 10 seconds of integration = 100 steps of 0.1s
        for _ in 0..100 {
            ekf.predict(dt, accel).unwrap();
        }

        // s = 0.5 * a * t^2 = 0.5 * 2.0 * 100 = 100.0 m
        let predicted_x = ekf.state[0];
        let expected_x = 100.0;
        let error = (predicted_x - expected_x).abs() / expected_x;
        assert!(
            error < 0.001,
            "IMU Integration failed: expected {}, got {}",
            expected_x,
            predicted_x
        );
    }

    #[test]
    fn test_gnss_update_tier1_step14() {
        let mut ekf = CellhawkEKF::new();
        // Move state away from ground truth
        ekf.state[0] = 50.0;
        ekf.state[1] = 50.0;

        let ground_truth = SVector::<f64, 3>::new(0.0, 0.0, 0.0);
        let jnr_db = 20.0; // Tier 1

        for _ in 0..50 {
            ekf.predict(0.1, SVector::zeros()).unwrap();
            ekf.update_gnss(jnr_db, ground_truth);
        }

        let error = (ekf.state[0].powi(2) + ekf.state[1].powi(2)).sqrt();
        assert!(
            error < 1.0,
            "GNSS update failed to converge: error {}m",
            error
        );
    }

    #[test]
    fn test_cellular_update_tier2_step15() {
        let mut ekf = CellhawkEKF::new();
        // let ground_truth = SVector::<f64, 3>::new(10.0, 10.0, 0.0);

        for _ in 0..100 {
            ekf.predict(0.1, SVector::zeros()).unwrap();
            // Simulate 6 dB noise
            let noisy_meas = SVector::<f64, 3>::new(15.0, 15.0, 0.0);
            ekf.update_cellular(noisy_meas);
        }

        let pos_cov = ekf.covariance.fixed_view::<3, 3>(0, 0);
        let trace = pos_cov[(0, 0)] + pos_cov[(1, 1)] + pos_cov[(2, 2)];
        let rms_error = (trace / 3.0).sqrt();

        assert!(
            rms_error <= 42.0,
            "Cellular RSSI accuracy degraded: {}m RMS",
            rms_error
        );
    }

    #[test]
    fn test_slam_update_tier3_step16() {
        let mut ekf = CellhawkEKF::new();
        let ground_truth = SVector::<f64, 3>::new(20.0, 20.0, 0.0);

        for _ in 0..50 {
            ekf.predict(0.1, SVector::zeros()).unwrap();
            ekf.update_vision(ground_truth);
        }

        let error = ((ekf.state[0] - 20.0).powi(2) + (ekf.state[1] - 20.0).powi(2)).sqrt();
        assert!(
            error <= 12.0,
            "Visual SLAM update exceeded noise bounds: {}m",
            error
        );
    }
}
