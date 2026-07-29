use crate::types::IntentVector;
use log::error;
use nalgebra::{DMatrix, DVector};

/// CORTEX DQN Model Port (Pure Rust Forward Pass)
/// Represents the ~10,000 parameter DQN described in the paper.
pub struct CortexDQNPolicy {
    w1: DMatrix<f64>,
    b1: DVector<f64>,
    w2: DMatrix<f64>,
    b2: DVector<f64>,
    w3: DMatrix<f64>,
    b3: DVector<f64>,

    actions: Vec<(f64, f64)>, // (dh, dz)
}

impl CortexDQNPolicy {
    pub fn new() -> Self {
        // Initialize with zeros or randoms. In production, we deserialize from a saved ONNX/pt model.
        let w1 = DMatrix::zeros(19, 64);
        let b1 = DVector::zeros(64);
        let w2 = DMatrix::zeros(64, 64);
        let b2 = DVector::zeros(64);
        let w3 = DMatrix::zeros(64, 15);
        let b3 = DVector::zeros(15);

        let mut actions = Vec::with_capacity(15);
        let dh_vals = [
            -std::f64::consts::PI / 4.0,
            -std::f64::consts::PI / 8.0,
            0.0,
            std::f64::consts::PI / 8.0,
            std::f64::consts::PI / 4.0,
        ];
        let dz_vals = [-2.0, 0.0, 2.0];

        for dh in &dh_vals {
            for dz in &dz_vals {
                actions.push((*dh, *dz));
            }
        }

        Self {
            w1,
            b1,
            w2,
            b2,
            w3,
            b3,
            actions,
        }
    }

    fn relu(mut v: DVector<f64>) -> DVector<f64> {
        for val in v.iter_mut() {
            if *val < 0.0 {
                *val = 0.0;
            }
        }
        v
    }

    pub fn forward(&self, state_19d: &DVector<f64>) -> (usize, DVector<f64>) {
        if state_19d.nrows() != 19 {
            error!("Cortex state must be 19-dimensional");
            return (0, DVector::zeros(64));
        }

        let h1 = Self::relu(self.w1.transpose() * state_19d + &self.b1);
        let h2 = Self::relu(self.w2.transpose() * &h1 + &self.b2);
        let q_values = self.w3.transpose() * &h2 + &self.b3;

        let mut best_action = 0;
        let mut best_q = f64::NEG_INFINITY;

        for i in 0..15 {
            if q_values[i] > best_q {
                best_q = q_values[i];
                best_action = i;
            }
        }

        (best_action, h2)
    }

    pub fn get_intent(
        &self,
        state_19d: &DVector<f64>,
        current_heading: f64,
        cruise_speed: f64,
    ) -> IntentVector {
        let (action_idx, _) = self.forward(state_19d);
        let (dh, dz) = self.actions[action_idx];

        let mut target_heading = current_heading + dh;
        let two_pi = 2.0 * std::f64::consts::PI;
        if target_heading > two_pi {
            target_heading -= two_pi;
        }
        if target_heading < 0.0 {
            target_heading += two_pi;
        }

        IntentVector {
            target_heading_rad: target_heading,
            target_climb_rate_mps: dz,
            target_speed_mps: cruise_speed,
        }
    }
}
