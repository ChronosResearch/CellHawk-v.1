use cellhawk_core::ekf::EKFNavigationEngine;
use log::{info, error};
use serde::{Deserialize, Serialize};
use zmq::Context;

#[derive(Deserialize, Debug)]
struct SensorData {
    jnr_db: f64,
    gnss_pos: Option<[f64; 3]>,
    cell_pos: Option<[f64; 3]>,
    vslam_pos: Option<[f64; 3]>,
    control_accel: Option<[f64; 3]>,
}

#[derive(Serialize, Debug)]
struct TelemetryOut {
    position: [f64; 3],
    velocity: [f64; 3],
    current_tier: String,
    rms_error: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    info!("Starting CELLHAWK Rust EKF Engine...");

    let ctx = Context::new();
    
    // ZMQ PULL socket for incoming sensor data from Python/C++
    let receiver = ctx.socket(zmq::PULL)?;
    receiver.bind("tcp://0.0.0.0:5555")?;
    info!("EKF Engine listening for sensor data on tcp://0.0.0.0:5555");

    // ZMQ PUB socket for outgoing telemetry
    let publisher = ctx.socket(zmq::PUB)?;
    publisher.bind("tcp://0.0.0.0:5556")?;
    info!("EKF Engine publishing telemetry on tcp://0.0.0.0:5556");

    let mut ekf = EKFNavigationEngine::new(0.1, 10.0, 20.0, 5);

    loop {
        let msg = receiver.recv_string(0)?;
        if let Ok(data) = msg {
            match serde_json::from_str::<SensorData>(&data) {
                Ok(sensor) => {
                    let gnss = sensor.gnss_pos.map(|v| nalgebra::Vector3::new(v[0], v[1], v[2]));
                    let cell = sensor.cell_pos.map(|v| nalgebra::Vector3::new(v[0], v[1], v[2]));
                    let vslam = sensor.vslam_pos.map(|v| nalgebra::Vector3::new(v[0], v[1], v[2]));
                    let accel = sensor.control_accel.map(|v| nalgebra::Vector3::new(v[0], v[1], v[2]));

                    match ekf.step(sensor.jnr_db, gnss, cell, vslam, accel) {
                        Ok(state) => {
                            let telemetry = TelemetryOut {
                                position: state.position,
                                velocity: state.velocity,
                                current_tier: format!("{:?}", state.current_tier),
                                rms_error: state.estimated_rms_error_m,
                            };
                            let out_str = serde_json::to_string(&telemetry)?;
                            publisher.send(&out_str, 0)?;
                        }
                        Err(e) => error!("EKF Update Failed: {:?}", e),
                    }
                }
                Err(e) => error!("Failed to deserialize sensor data: {}", e),
            }
        }
    }
}
