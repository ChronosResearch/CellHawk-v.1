use mavlink::{Message, common::*};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;
use tracing::{info, error};

pub struct MavlinkProxy {
    connection: Arc<Mutex<Box<dyn mavlink::MavConnection<MavMessage> + Send + Sync>>>,
}

impl MavlinkProxy {
    pub fn new(address: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let conn = mavlink::connect(address)?;
        Ok(Self {
            connection: Arc::new(Mutex::new(conn)),
        })
    }

    pub async fn send_attitude_target(
        &self,
        roll: f32,
        pitch: f32,
        yaw: f32,
        thrust: f32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let header = mavlink::MavHeader::default();
        
        let q = euler_to_quaternion(roll, pitch, yaw);
        
        let msg = MavMessage::SET_ATTITUDE_TARGET(SET_ATTITUDE_TARGET_DATA {
            time_boot_ms: 0,
            q,
            body_roll_rate: 0.0,
            body_pitch_rate: 0.0,
            body_yaw_rate: 0.0,
            thrust,
            target_system: 1,
            target_component: 1,
            type_mask: 0b111, // Ignore rates
        });

        let mut conn = self.connection.lock().await;
        conn.send(&header, &msg)?;
        Ok(())
    }
}

fn euler_to_quaternion(roll: f32, pitch: f32, yaw: f32) -> [f32; 4] {
    let cr = (roll * 0.5).cos();
    let sr = (roll * 0.5).sin();
    let cp = (pitch * 0.5).cos();
    let sp = (pitch * 0.5).sin();
    let cy = (yaw * 0.5).cos();
    let sy = (yaw * 0.5).sin();

    [
        cr * cp * cy + sr * sp * sy, // w
        sr * cp * cy - cr * sp * sy, // x
        cr * sp * cy + sr * cp * sy, // y
        cr * cp * sy - sr * sp * cy, // z
    ]
}
