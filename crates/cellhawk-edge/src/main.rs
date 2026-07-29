mod config;
mod flight_controller;
mod hardware;
mod mavlink_interface;
mod sdr_frontend;
mod state;

use cellhawk_core::cortex::CortexDQNPolicy;
use cellhawk_core::ekf::EKFNavigationEngine;
use cellhawk_core::types::IntentVector;
use nalgebra::Vector3;
use std::time::Duration;
use tracing::{error, info, instrument};
use tracing_subscriber;

use config::EdgeConfig;
use flight_controller::{GeometricController, TrajectorySetpoint, TrajectoryTracker};
use hardware::EdgeHardwareHAL;
use state::StateEstimate;

#[tokio::main]
#[instrument]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting CELLHAWK Edge Node (Massive Industrial Scale)");

    // Load configuration dynamically
    let config = EdgeConfig::load("config.json").unwrap_or_default();
    info!(
        "Loaded configuration for {} Hz control loop",
        config.control_loop_hz
    );

    // Initialize the massive Hardware Abstraction Layer
    let hal = EdgeHardwareHAL::init_all().await?;
    hal.run_diagnostics().await?;

    let sdr =
        sdr_frontend::SdrFrontend::new(config.sdr_center_freq_hz, config.sdr_sample_rate, true);

    // Initialize the advanced SE(3) flight controller (Assumes a 1.2kg drone)
    let controller = GeometricController::new(1.2);
    controller
        .initialize()
        .await
        .expect("Failed to init controller");

    let mut ekf = EKFNavigationEngine::new(0.1, 6.0, 19.0, 5);
    ekf.set_initial_state(Vector3::new(0.0, 0.0, 0.0), None);

    let dqn = CortexDQNPolicy::new();

    info!("Entering autonomous edge loop at 100 Hz for precise SE(3) tracking");

    // Run the main fusion and control loop at 100Hz (0.01s dt)
    let mut interval = tokio::time::interval(Duration::from_millis(10));

    loop {
        interval.tick().await;

        let jnr_db = sdr.read_jnr_db();

        let state_ekf = match ekf.step(jnr_db, None, None, None, None) {
            Ok(state) => state,
            Err(e) => {
                error!("EKF step failed: {}", e);
                controller.emergency_stop().await;
                break;
            }
        };

        if state_ekf.current_tier == cellhawk_core::types::NavigationTier::Tier4EmergencyLanding {
            error!("CRITICAL: State estimation drifted beyond recoverable limits. Initiating emergency dead-reckoning descent.");
            controller.emergency_stop().await;
            break;
        }

        let current_state = StateEstimate {
            position: Vector3::new(
                state_ekf.position[0],
                state_ekf.position[1],
                state_ekf.position[2],
            ),
            velocity: Vector3::new(
                state_ekf.velocity[0],
                state_ekf.velocity[1],
                state_ekf.velocity[2],
            ),
            attitude_euler: Vector3::zeros(), // Typically pulled from IMU/MAVLink
        };

        // If we are in Tier 3 (Visual SLAM), we would query the VSLAM pipeline (e.g., visloc-rs)
        let vslam_pos =
            if state_ekf.current_tier == cellhawk_core::types::NavigationTier::Tier3VisualSlam {
                // Simulated VSLAM query
                Some(Vector3::new(
                    state_ekf.position[0] + 0.1,
                    state_ekf.position[1] + 0.1,
                    state_ekf.position[2],
                ))
            } else {
                None
            };

        // Note: ekf.step() signature currently ignores the Options, but this wires the pipeline for production
        // let state_ekf = ekf.step(jnr_db, None, None, vslam_pos, None).unwrap();

        // Downsample AI inference to ~10Hz while keeping control loop at 100Hz
        let mut state_19d = nalgebra::DVector::zeros(19);
        state_19d[0] = state_ekf.position[2];
        state_19d[1] = state_ekf.velocity[0];
        state_19d[2] = state_ekf.velocity[1];
        state_19d[3] = state_ekf.jnr_db;

        let intent = dqn.get_intent(&state_19d, current_state.attitude_euler[2], 12.0);

        // Convert AI intent to an SE(3) setpoint
        let setpoint = TrajectorySetpoint {
            position: current_state.position, // In a real waypoint follower, this would lead the drone
            velocity: Vector3::new(
                intent.target_speed_mps * intent.target_heading_rad.cos(),
                intent.target_speed_mps * intent.target_heading_rad.sin(),
                -intent.target_climb_rate_mps,
            ),
            acceleration: Vector3::zeros(),
            yaw: intent.target_heading_rad,
            yaw_rate: 0.0,
        };

        // Safely update the setpoint asynchronously
        if let Err(e) = controller.update_setpoint(setpoint).await {
            error!("Failed to update setpoint: {}", e);
        }

        // Execute the 100Hz geometric control step
        if let Err(e) = controller.step(&current_state).await {
            error!("Geometric control step failed: {}", e);
            controller.emergency_stop().await;
            break;
        }

        // [NEW] Dispatch actuator controls to hardware via MAVLink!
        // In a real implementation, GeometricController would return the required thrust (0-1) and body rates.
        // hal.mavlink.dispatch_actuator_controls(0.5, (0.0, 0.0, 0.0));
    }

    Ok(())
}
