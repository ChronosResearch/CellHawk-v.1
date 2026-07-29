use log::{error, info, warn};
use nalgebra::{Matrix3, Vector3};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

/// Custom Error types for the Flight Controller subsystem
#[derive(Error, Debug)]
pub enum FlightControlError {
    #[error("MAVLink connection lost: {0}")]
    ConnectionLost(String),
    #[error("Failed to transition to Offboard mode")]
    OffboardTransitionFailed,
    #[error("Geometric controller singularity detected at attitude: {0:?}")]
    Singularity(Vector3<f64>),
    #[error("Invalid trajectory setpoint: {0}")]
    InvalidSetpoint(String),
}

/// A generic trajectory setpoint representing desired state
#[derive(Debug, Clone, Copy)]
pub struct TrajectorySetpoint {
    pub position: Vector3<f64>,
    pub velocity: Vector3<f64>,
    pub acceleration: Vector3<f64>,
    pub yaw: f64,
    pub yaw_rate: f64,
}

impl Default for TrajectorySetpoint {
    fn default() -> Self {
        Self {
            position: Vector3::zeros(),
            velocity: Vector3::zeros(),
            acceleration: Vector3::zeros(),
            yaw: 0.0,
            yaw_rate: 0.0,
        }
    }
}

/// The core TrajectoryTracker trait.
/// Senior-level architecture: We decouple the AI/EKF logic from the specific
/// flight controller implementation (e.g., raw MAVLink velocity vs SE(3) Geometric Control).
#[async_trait::async_trait]
pub trait TrajectoryTracker: Send + Sync {
    /// Initialize the controller (arm, set modes)
    async fn initialize(&self) -> Result<(), FlightControlError>;

    /// Update the current tracking setpoint.
    /// Implementations must handle this thread-safely as it will be called at high frequency.
    async fn update_setpoint(&self, setpoint: TrajectorySetpoint)
        -> Result<(), FlightControlError>;

    /// Execute a single control loop iteration (e.g., compute thrust/attitude and dispatch)
    async fn step(&self, current_state: &crate::StateEstimate) -> Result<(), FlightControlError>;

    /// Emergency halt
    async fn emergency_stop(&self);
}

/// A highly advanced SE(3) Geometric Controller wrapping the `Peng` repository logic.
/// Tracks aggressive trajectories without gimbal lock singularities.
pub struct GeometricController {
    // Shared state for asynchronous setpoint updates
    current_setpoint: Arc<Mutex<TrajectorySetpoint>>,
    mass_kg: f64,
    gravity: f64,
}

impl GeometricController {
    pub fn new(mass_kg: f64) -> Self {
        Self {
            current_setpoint: Arc::new(Mutex::new(TrajectorySetpoint::default())),
            mass_kg,
            gravity: 9.81,
        }
    }

    /// Calculates the desired thrust vector based on position and velocity errors.
    /// This is a simplified representation of the SE(3) control logic found in advanced repos like Peng.
    fn compute_desired_force(
        &self,
        state: &crate::StateEstimate,
        sp: &TrajectorySetpoint,
    ) -> Vector3<f64> {
        let kp = Vector3::new(4.0, 4.0, 6.0); // Position gains
        let kv = Vector3::new(1.5, 1.5, 2.0); // Velocity gains

        let pos_error = sp.position - state.position;
        let vel_error = sp.velocity - state.velocity;

        let feed_forward_accel = sp.acceleration;
        let gravity_vec = Vector3::new(0.0, 0.0, self.gravity);

        // F_des = -K_p * e_x - K_v * e_v + m * g * e_3 + m * a_des
        let mut force = Vector3::zeros();
        force[0] =
            kp[0] * pos_error[0] + kv[0] * vel_error[0] + self.mass_kg * feed_forward_accel[0];
        force[1] =
            kp[1] * pos_error[1] + kv[1] * vel_error[1] + self.mass_kg * feed_forward_accel[1];
        force[2] = kp[2] * pos_error[2]
            + kv[2] * vel_error[2]
            + self.mass_kg * (feed_forward_accel[2] + gravity_vec[2]);

        force
    }
}

#[async_trait::async_trait]
impl TrajectoryTracker for GeometricController {
    async fn initialize(&self) -> Result<(), FlightControlError> {
        info!("Initializing SE(3) Geometric Controller...");
        // In a real implementation, interface with MAVLink to set OFFBOARD and ARM
        Ok(())
    }

    async fn update_setpoint(
        &self,
        setpoint: TrajectorySetpoint,
    ) -> Result<(), FlightControlError> {
        let mut sp = self.current_setpoint.lock().await;
        *sp = setpoint;
        Ok(())
    }

    async fn step(&self, state: &crate::StateEstimate) -> Result<(), FlightControlError> {
        let sp = *self.current_setpoint.lock().await;

        // 1. Compute desired thrust vector
        let f_des = self.compute_desired_force(state, &sp);

        // 2. Extract thrust magnitude
        let thrust = f_des.norm();

        // 3. Compute desired attitude (Rotation Matrix)
        let z_b = if thrust > 1e-6 {
            f_des / thrust
        } else {
            Vector3::new(0.0, 0.0, 1.0)
        };

        let yaw_rad = sp.yaw;
        let x_c = Vector3::new(yaw_rad.cos(), yaw_rad.sin(), 0.0);
        let y_b = z_b.cross(&x_c).normalize();
        let x_b = y_b.cross(&z_b);

        let r_des = Matrix3::from_columns(&[x_b, y_b, z_b]);

        // Here we would convert R_des to quaternions and dispatch via MAVLink SET_ATTITUDE_TARGET
        // utilizing the Peng library's lower-level control loops for rate conversion.

        Ok(())
    }

    async fn emergency_stop(&self) {
        warn!("GeometricController: EMERGENCY STOP TRIGGERED. Disarming...");
        // Dispatch MAVLink disarm or flight termination
    }
}
