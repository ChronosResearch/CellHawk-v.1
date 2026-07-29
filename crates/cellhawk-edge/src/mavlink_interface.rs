use cellhawk_core::types::IntentVector;
use log::{error, info, warn};
use mavlink::common::MavMessage;
use mavlink::{MavConnection, MavHeader};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Robust MAVLink Interface for PX4 Offboard Control.
/// Implements a dedicated background thread to guarantee the 20Hz+ setpoint streaming
/// required by PX4 to maintain Offboard mode without triggering failsafes.
pub struct MavlinkInterface {
    conn: Arc<Mutex<Box<dyn MavConnection<MavMessage> + Send + Sync>>>,
    pub system_id: u8,
    pub component_id: u8,

    // Shared state for the background streaming thread
    target_velocity: Arc<Mutex<(f32, f32, f32, f32)>>, // vx, vy, vz, yaw
    streaming_active: Arc<Mutex<bool>>,
}

impl MavlinkInterface {
    pub fn new(connection_string: &str) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Connecting to MAVLink on {}", connection_string);
        let conn: Box<dyn MavConnection<MavMessage> + Send + Sync> =
            mavlink::connect(connection_string)?;
        let conn = Arc::new(Mutex::new(conn));

        let interface = Self {
            conn,
            system_id: 255, // GCS / Companion Computer system ID
            component_id: 0,
            target_velocity: Arc::new(Mutex::new((0.0, 0.0, 0.0, 0.0))),
            streaming_active: Arc::new(Mutex::new(false)),
        };

        interface.wait_for_heartbeat(Duration::from_secs(10))?;

        // Start the background high-frequency streaming thread
        interface.start_setpoint_streamer();

        Ok(interface)
    }

    fn wait_for_heartbeat(&self, timeout: Duration) -> Result<(), Box<dyn std::error::Error>> {
        let start = Instant::now();
        loop {
            if start.elapsed() > timeout {
                return Err("Timeout waiting for PX4 heartbeat".into());
            }

            let conn = self.conn.lock().unwrap();
            if let Ok((_header, msg)) = conn.recv() {
                if let MavMessage::HEARTBEAT(hb) = msg {
                    info!(
                        "PX4 Heartbeat received. Autopilot: {:?}, Base Mode: {:?}",
                        hb.autopilot, hb.base_mode
                    );
                    return Ok(());
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    /// Spawns a dedicated thread to stream setpoints at 20Hz.
    /// PX4 requires a continuous stream of setpoints before entering OFFBOARD mode,
    /// and if the stream drops below ~2Hz, it will trigger an offboard loss failsafe.
    fn start_setpoint_streamer(&self) {
        let conn_clone = Arc::clone(&self.conn);
        let vel_clone = Arc::clone(&self.target_velocity);
        let active_clone = Arc::clone(&self.streaming_active);
        let sys_id = self.system_id;

        // Mark as active
        *active_clone.lock().unwrap() = true;

        thread::spawn(move || {
            info!("MAVLink 20Hz Setpoint Streamer Thread Started.");
            let loop_interval = Duration::from_millis(50); // 20Hz

            let type_mask = (1 << 0) | (1 << 1) | (1 << 2) | // Ignore position (x, y, z)
                (1 << 6) | (1 << 7) | (1 << 8) | // Ignore acceleration (afx, afy, afz)
                (1 << 11); // Ignore yaw rate

            loop {
                let start_loop = Instant::now();

                if !*active_clone.lock().unwrap() {
                    break;
                }

                let (vx, vy, vz, yaw) = *vel_clone.lock().unwrap();

                let msg = MavMessage::SET_POSITION_TARGET_LOCAL_NED(
                    mavlink::common::SET_POSITION_TARGET_LOCAL_NED_DATA {
                        time_boot_ms: 0,
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                        vx,
                        vy,
                        vz,
                        afx: 0.0,
                        afy: 0.0,
                        afz: 0.0,
                        yaw,
                        yaw_rate: 0.0,
                        type_mask,
                        target_system: 1, // Usually 1 for the drone
                        target_component: 1,
                        coordinate_frame: mavlink::common::MavFrame::MAV_FRAME_LOCAL_NED,
                    },
                );

                let header = MavHeader::get_default_header();

                // Also send a heartbeat from the companion computer
                let hb_msg = MavMessage::HEARTBEAT(mavlink::common::HEARTBEAT_DATA {
                    custom_mode: 0,
                    type_: mavlink::common::MavType::MAV_TYPE_ONBOARD_CONTROLLER,
                    autopilot: mavlink::common::MavAutopilot::MAV_AUTOPILOT_INVALID,
                    base_mode: mavlink::common::MavModeFlag::empty(),
                    system_status: mavlink::common::MavState::MAV_STATE_ACTIVE,
                    mavlink_version: 3,
                });

                if let Ok(mut c) = conn_clone.lock() {
                    let _ = c.send(&header, &msg);
                    let _ = c.send(&header, &hb_msg);
                }

                let elapsed = start_loop.elapsed();
                if elapsed < loop_interval {
                    thread::sleep(loop_interval - elapsed);
                }
            }
            warn!("MAVLink Setpoint Streamer Thread Stopped.");
        });
    }

    pub fn arm(&self, target_sys: u8, target_comp: u8) -> Result<(), Box<dyn std::error::Error>> {
        info!("Sending ARM command to PX4...");
        let msg = MavMessage::COMMAND_LONG(mavlink::common::COMMAND_LONG_DATA {
            param1: 1.0, // 1 = ARM
            param2: 0.0,
            param3: 0.0,
            param4: 0.0,
            param5: 0.0,
            param6: 0.0,
            param7: 0.0,
            command: mavlink::common::MavCmd::MAV_CMD_COMPONENT_ARM_DISARM,
            target_system: target_sys,
            target_component: target_comp,
            confirmation: 0,
        });

        let header = MavHeader::get_default_header();
        self.conn.lock().unwrap().send(&header, &msg)?;
        Ok(())
    }

    pub fn set_offboard_mode(
        &self,
        target_sys: u8,
        target_comp: u8,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Requesting PX4 OFFBOARD mode transition...");

        // Ensure we've been streaming for a bit before requesting
        thread::sleep(Duration::from_millis(200));

        let msg = MavMessage::COMMAND_LONG(mavlink::common::COMMAND_LONG_DATA {
            param1: mavlink::common::MavModeFlag::MAV_MODE_FLAG_CUSTOM_MODE_ENABLED.bits() as f32,
            param2: 1.0, // PX4 custom main mode 1 (Offboard)
            param3: 6.0, // PX4 sub-mode
            param4: 0.0,
            param5: 0.0,
            param6: 0.0,
            param7: 0.0,
            command: mavlink::common::MavCmd::MAV_CMD_DO_SET_MODE,
            target_system: target_sys,
            target_component: target_comp,
            confirmation: 0,
        });

        let header = MavHeader::get_default_header();
        self.conn.lock().unwrap().send(&header, &msg)?;
        Ok(())
    }

    /// Updates the target velocity shared state. The background thread will automatically
    /// stream this to the PX4 at 20Hz.
    pub fn dispatch_cortex_intent(&self, intent: &IntentVector) {
        // Convert Intent (heading, speed, climb) into Local NED velocity
        let vx = (intent.target_speed_mps * intent.target_heading_rad.cos()) as f32;
        let vy = (intent.target_speed_mps * intent.target_heading_rad.sin()) as f32;
        let vz = (-intent.target_climb_rate_mps) as f32; // Down is positive in NED
        let yaw = intent.target_heading_rad as f32;

        let mut vel = self.target_velocity.lock().unwrap();
        *vel = (vx, vy, vz, yaw);
    }

    pub fn dispatch_actuator_controls(&self, thrust_normalized: f32, body_rates: (f32, f32, f32)) {
        // Send SET_ATTITUDE_TARGET to PX4
        let msg = MavMessage::SET_ATTITUDE_TARGET(mavlink::common::SET_ATTITUDE_TARGET_DATA {
            time_boot_ms: 0,
            target_system: self.system_id,
            target_component: self.component_id,
            type_mask: 128, // 128 = Ignore attitude, only use rates and thrust
            q: [1.0, 0.0, 0.0, 0.0], // Ignored due to mask
            body_roll_rate: body_rates.0,
            body_pitch_rate: body_rates.1,
            body_yaw_rate: body_rates.2,
            thrust: thrust_normalized,
        });

        let header = MavHeader::get_default_header();
        if let Ok(mut c) = self.conn.lock() {
            let _ = c.send(&header, &msg);
        }
    }

    pub fn stop(&self) {
        *self.streaming_active.lock().unwrap() = false;
    }
}
