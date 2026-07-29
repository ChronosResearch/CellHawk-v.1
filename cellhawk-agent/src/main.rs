use std::error::Error;
use std::net::UdpSocket;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::time::{self, Duration, Instant};
use tracing::{error, info, warn};

pub mod swarm;

fn check_secure_permissions(path: &str) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(path) {
            let mode = meta.permissions().mode();
            if mode & 0o077 != 0 {
                warn!(
                    "SECURITY ALERT: {} has unsafe permissions ({:o}). Expected 600.",
                    path, mode
                );
            }
        }
    }
    #[cfg(not(unix))]
    {
        warn!(
            "OS does not support Unix permissions. Skipping permission check for {}.",
            path
        );
    }
}

fn setup_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        error!(
            "FATAL PANIC: {}. Attempting graceful zeroize of EKF state.",
            info
        );
        // In a real scenario, we would cleanly shut down actuators here.
    }));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();
    setup_panic_hook();
    info!("Starting CELLHAWK Agent (Orchestrator)...");

    // Simulate config loading with ENV fallbacks
    let _drand_url =
        std::env::var("CELLHAWK_DRAND_URL").unwrap_or_else(|_| "https://api.drand.sh".to_string());

    check_secure_permissions("config.toml");
    check_secure_permissions("towers.json");

    // Channels
    let (sdr_tx, mut sdr_rx) = mpsc::channel(32);

    // 1. Spawn SDR Task
    tokio::spawn(async move {
        // Simulated SDR task
        cellhawk_sdr::run_sdr_telemetry(sdr_tx).await;
    });

    // 2. Spawn Vision Task
    tokio::spawn(async move {
        // Simulated Vision task pushing images to ORB-SLAM2
        let mut interval = time::interval(Duration::from_millis(50));
        loop {
            interval.tick().await;
            // Vision processing mock
        }
    });

    // 3. MAVLink / PID Translation Task
    let (mav_tx, mut mav_rx) = mpsc::channel::<(f32, f32)>(32);
    tokio::spawn(async move {
        let mut telemetry_pub = zmq::Context::new().socket(zmq::PUB)?;
        telemetry_pub
            .set_sndhwm(1000)
            .expect("Failed to set ZMQ HWM (Step 22)");
        telemetry_pub.bind("tcp://127.0.0.1:5555")?;

        while let Some((heading, climb)) = mav_rx.recv().await {
            // PID Translation (Mock)
            let roll = 0.0;
            let pitch = climb * 0.1;
            let yaw = heading * 3.14159 / 180.0;
            let thrust = 0.5;

            // Send mock SET_ATTITUDE MAVLink frame
            let msg = format!(
                "MAV: roll={} pitch={} yaw={} thrust={}",
                roll, pitch, yaw, thrust
            );
            let _ = socket.send(msg.as_bytes());
        }
    });

    // EKF Setup
    let mut ekf = cellhawk_ekf::CellhawkEKF::new();
    let mut interval = time::interval(Duration::from_millis(100));

    let mut jnr_stopwatch: Option<Instant> = None;

    loop {
        let loop_start = Instant::now();

        tokio::select! {
            _ = interval.tick() => {
                let simulated_jnr = 5.0; // Simulated transition < 6dB

                // JNR Handover Timer Logic
                if simulated_jnr < 6.0 && jnr_stopwatch.is_none() {
                    jnr_stopwatch = Some(Instant::now());
                }

                // Predict & Update EKF
                let _ = ekf.predict(0.1, nalgebra::SVector::zeros());
                ekf.update_gnss(simulated_jnr, nalgebra::SVector::zeros());

                // Check if transition to Tier 2/3 is complete (target_scale = 100.0)
                if simulated_jnr < 6.0 {
                    // if current scale has reached target scale
                    if let Some(start) = jnr_stopwatch {
                        let elapsed = start.elapsed().as_millis();
                        // Assume 5 cycles (500ms) for interpolation to finish
                        if elapsed >= 500 {
                            if elapsed > 250 {
                                tracing::error!("CRITICAL: Handover latency exceeded bounds: {} ms", elapsed);
                            }
                            jnr_stopwatch = None; // Reset
                        }
                    }
                }

                // Mock DQN intent calculation (pass to MAVLink)
                let _ = mav_tx.try_send((45.0, 1.0));

                let loop_time = loop_start.elapsed().as_millis();
                if loop_time > 120 {
                    tracing::warn!("WARNING: EKF Loop exceeded 120ms ({}ms)", loop_time);
                }
            }
            Some(rssi) = sdr_rx.recv() => {
                // Process incoming SDR telemetry
                let _range = cellhawk_sdr::rssi_to_range(rssi, -20.0, 2.8, 1.0);
            }
        }
    }
}
