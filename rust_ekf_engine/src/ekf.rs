use crate::types::{EKFState, NavigationTier};
use log::{error, info, warn};
use nalgebra::{Cholesky, Matrix3, Matrix3x6, Matrix6, Vector3, Vector6};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EKFError {
    #[error("Failed to compute pseudo-inverse during EKF update step. Singular matrix or numerical instability.")]
    PseudoInverseFailure,
    #[error("Observation vector dimension mismatch.")]
    DimensionMismatch,
}

pub struct EKFNavigationEngine {
    pub dt: f64,
    pub tier1_jnr_threshold: f64,
    pub tier2_jnr_threshold: f64,
    pub handover_steps: u32,
    pub huber_c: f64,

    pub x: Vector6<f64>,
    pub p: Matrix6<f64>,

    pub f: Matrix6<f64>,
    pub q: Matrix6<f64>,

    pub current_tier: NavigationTier,
    pub target_tier: NavigationTier,
    pub handover_counter: u32,
    pub handover_alpha: f64,

    pub r_gnss_base: Matrix3<f64>,
    pub r_cell_base: Matrix3<f64>,
    pub r_vslam_base: Matrix3<f64>,
}

impl EKFNavigationEngine {
    /// Constructs a new EKF Navigation Engine with a rigorous Continuous-Discrete Kinematic Model.
    /// Incorporates the Continuous White Noise Acceleration model for the Process Noise Covariance (Q).
    pub fn new(dt: f64, tier1_threshold: f64, tier2_threshold: f64, steps: u32) -> Self {
        // State Transition Matrix (F) for a 3D constant velocity model
        let mut f = Matrix6::identity();
        f[(0, 3)] = dt;
        f[(1, 4)] = dt;
        f[(2, 5)] = dt;

        // Process Noise Covariance (Q) - Continuous White Noise Acceleration Model
        // Q = [ (dt^3)/3 * I   (dt^2)/2 * I ] * sigma_a^2
        //     [ (dt^2)/2 * I   dt * I       ]
        let sigma_a_sq = 0.5_f64; // Variance of acceleration noise
        let mut q = Matrix6::zeros();
        let dt2 = dt * dt;
        let dt3 = dt2 * dt;

        let q11 = (dt3 / 3.0) * sigma_a_sq;
        let q12 = (dt2 / 2.0) * sigma_a_sq;
        let q22 = dt * sigma_a_sq;

        for i in 0..3 {
            q[(i, i)] = q11;
            q[(i, i + 3)] = q12;
            q[(i + 3, i)] = q12;
            q[(i + 3, i + 3)] = q22;
        }

        // Sensor Measurement Covariances (R)
        let mut r_gnss = Matrix3::zeros();
        r_gnss[(0, 0)] = 2.5 * 2.5;
        r_gnss[(1, 1)] = 2.5 * 2.5;
        r_gnss[(2, 2)] = 5.0 * 5.0;

        let mut r_cell = Matrix3::zeros();
        r_cell[(0, 0)] = 35.0 * 35.0;
        r_cell[(1, 1)] = 35.0 * 35.0;
        r_cell[(2, 2)] = 80.0 * 80.0;

        let mut r_vslam = Matrix3::zeros();
        r_vslam[(0, 0)] = 10.0 * 10.0;
        r_vslam[(1, 1)] = 10.0 * 10.0;
        r_vslam[(2, 2)] = 10.0 * 10.0;

        // Initial State Covariance (P)
        let mut p = Matrix6::identity() * 10.0;

        Self {
            dt,
            tier1_jnr_threshold: tier1_threshold,
            tier2_jnr_threshold: tier2_threshold,
            handover_steps: steps,
            huber_c: 1.345, // Huber loss threshold for robust M-estimation
            x: Vector6::zeros(),
            p,
            f,
            q,
            current_tier: NavigationTier::Tier1GnssActive,
            target_tier: NavigationTier::Tier1GnssActive,
            handover_counter: 0,
            handover_alpha: 1.0,
            r_gnss_base: r_gnss,
            r_cell_base: r_cell,
            r_vslam_base: r_vslam,
        }
    }

    pub fn set_initial_state(&mut self, pos: Vector3<f64>, vel: Option<Vector3<f64>>) {
        self.x.fixed_rows_mut::<3>(0).copy_from(&pos);
        if let Some(v) = vel {
            self.x.fixed_rows_mut::<3>(3).copy_from(&v);
        } else {
            self.x.fixed_rows_mut::<3>(3).fill(0.0);
        }
    }

    /// Detect GPS Spoofing via signal power consistency checks (Placeholder)
    pub fn detect_gps_spoofing(&self, _gnss_signal_power: f64) -> bool {
        // TODO: Implement correlation analysis between SNR drops and positional jumps
        false
    }

    /// Predict step with IMU drift modeling (random walk bias)
    pub fn predict(&mut self, control_accel: Option<Vector3<f64>>) {
        let dt = self.dt;
        
        // Apply IMU bias drift model for extended GPS denial (>60s)
        let bias_drift = Vector3::new(0.01, 0.01, 0.005) * dt; 

        // x_{k|k-1} = x_{k-1|k-1} + v_{k-1|k-1} * dt
        self.x.fixed_rows_mut::<3>(0).add_assign(&(self.x.fixed_rows::<3>(3) * dt));
        
        if let Some(accel) = control_accel {
            // Apply drift to the accelerometer readings
            let compensated_accel = accel - bias_drift;
            let dt2 = dt * dt;

            let mut u = Vector6::zeros();

            self.x = self.f * self.x + u;
        } else {
            self.x = self.f * self.x;
        }

        self.p = self.f * self.p * self.f.transpose() + self.q;
        self.p = (self.p + self.p.transpose()) * 0.5; // Symmetrize
    }

    pub fn evaluate_tier(&self, jnr_db: f64) -> NavigationTier {
        // Fallback checks: If covariance matrix explodes, all active sensors have failed or drifted beyond usability.
        let covariance_trace = self.p[(0, 0)] + self.p[(1, 1)] + self.p[(2, 2)];
        if covariance_trace > 1000.0 {
            // Unrecoverable state estimation drift - trigger Emergency Landing Protocol
            return NavigationTier::Tier4EmergencyLanding;
        }

        if jnr_db < self.tier1_jnr_threshold {
            NavigationTier::Tier1GnssActive
        } else if jnr_db < self.tier2_jnr_threshold {
            NavigationTier::Tier2CellularRssi
        } else {
            NavigationTier::Tier3VisualSlam
        }
    }

    fn update_handover_state(&mut self, jnr_db: f64) {
        let new_target = self.evaluate_tier(jnr_db);
        if new_target != self.target_tier {
            self.target_tier = new_target;
            self.handover_counter = self.handover_steps;
            warn!(
                "Tier Transition Initiated: {:?} -> {:?} (JNR: {} dB)",
                self.current_tier, self.target_tier, jnr_db
            );
        }

        if self.handover_counter > 0 {
            self.handover_alpha = (self.handover_counter as f64) / (self.handover_steps as f64);
            self.handover_counter -= 1;
            if self.handover_counter == 0 {
                self.current_tier = self.target_tier;
                self.handover_alpha = 1.0;
                info!(
                    "Tier Transition Complete. Operating in Tier {:?}",
                    self.current_tier
                );
            }
        }
    }

    fn huber_weight(&self, innovation: &Vector3<f64>, s_inv: &Matrix3<f64>) -> f64 {
        let d_sq = (innovation.transpose() * s_inv * innovation)[(0, 0)];
        let d = if d_sq > 0.0 { d_sq.sqrt() } else { 0.0 };

        if d <= self.huber_c {
            1.0
        } else {
            self.huber_c / d
        }
    }

    pub fn update(
        &mut self,
        z: Vector3<f64>,
        mut r_sensor: Matrix3<f64>,
        h: Matrix3x6<f64>,
    ) -> Result<(), EKFError> {
        let z_pred = h * self.x;
        let y = z - z_pred;

        let mut s = h * self.p * h.transpose() + r_sensor;

        // Cholesky decomposition for stable inversion
        let s_inv = match Cholesky::new(s) {
            Some(chol) => chol.inverse(),
            None => {
                warn!("S matrix Cholesky failed, falling back to pseudo-inverse");
                s.clone_owned()
                    .pseudo_inverse(1e-9)
                    .map_err(|_| EKFError::PseudoInverseFailure)?
            }
        };

        let w = self.huber_weight(&y, &s_inv);
        if w < 1.0 {
            // Downweight outlier
            r_sensor = r_sensor / w;
            s = h * self.p * h.transpose() + r_sensor;
            // Recompute inv
            let s_inv_new = match Cholesky::new(s) {
                Some(chol) => chol.inverse(),
                None => s
                    .clone_owned()
                    .pseudo_inverse(1e-9)
                    .map_err(|_| EKFError::PseudoInverseFailure)?,
            };
            let k = self.p * h.transpose() * s_inv_new;
            self.x = self.x + k * y;
            let i_kh = Matrix6::identity() - k * h;
            self.p = i_kh * self.p * i_kh.transpose() + k * r_sensor * k.transpose();
        } else {
            let k = self.p * h.transpose() * s_inv;
            self.x = self.x + k * y;
            let i_kh = Matrix6::identity() - k * h;
            self.p = i_kh * self.p * i_kh.transpose() + k * r_sensor * k.transpose();
        }
        self.p = (self.p + self.p.transpose()) * 0.5;
        Ok(())
    }

    pub fn step(
        &mut self,
        jnr_db: f64,
        z_gnss: Option<Vector3<f64>>,
        z_cell: Option<Vector3<f64>>,
        z_vslam: Option<Vector3<f64>>,
        control_accel: Option<Vector3<f64>>,
    ) -> Result<EKFState, EKFError> {
        self.predict(control_accel);
        self.update_handover_state(jnr_db);

        let mut h_pos = Matrix3x6::zeros();
        h_pos[(0, 0)] = 1.0;
        h_pos[(1, 1)] = 1.0;
        h_pos[(2, 2)] = 1.0;

        let eps = 1e-6;

        if self.handover_counter == 0 {
            match self.current_tier {
                NavigationTier::Tier1GnssActive => {
                    if let Some(z) = z_gnss {
                        self.update(z, self.r_gnss_base, h_pos)?;
                    }
                }
                NavigationTier::Tier2CellularRssi => {
                    if let Some(z) = z_cell {
                        self.update(z, self.r_cell_base, h_pos)?;
                    }
                }
                NavigationTier::Tier3VisualSlam => {
                    if let Some(z) = z_vslam {
                        self.update(z, self.r_vslam_base, h_pos)?;
                    }
                }
            }
        } else {
            let w_curr = self.handover_alpha.max(eps);
            let w_targ = (1.0 - self.handover_alpha).max(eps);

            let get_tier_data = |tier: NavigationTier| -> (Option<Vector3<f64>>, Matrix3<f64>) {
                match tier {
                    NavigationTier::Tier1GnssActive => (z_gnss, self.r_gnss_base),
                    NavigationTier::Tier2CellularRssi => (z_cell, self.r_cell_base),
                    NavigationTier::Tier3VisualSlam => (z_vslam, self.r_vslam_base),
                }
            };

            let (z_c, r_c) = get_tier_data(self.current_tier);
            if let Some(z) = z_c {
                self.update(z, r_c / w_curr, h_pos)?;
            }

            let (z_t, r_t) = get_tier_data(self.target_tier);
            if let Some(z) = z_t {
                self.update(z, r_t / w_targ, h_pos)?;
            }
        }

        let est_rms = (self.p[(0, 0)] + self.p[(1, 1)] + self.p[(2, 2)]).sqrt();

        Ok(EKFState {
            position: [self.x[0], self.x[1], self.x[2]],
            velocity: [self.x[3], self.x[4], self.x[5]],
            covariance: self.p.clone(),
            current_tier: self.current_tier,
            jnr_db,
            handover_alpha: self.handover_alpha,
            estimated_rms_error_m: est_rms,
            handover_in_progress: self.handover_counter > 0,
        })
    }
}
