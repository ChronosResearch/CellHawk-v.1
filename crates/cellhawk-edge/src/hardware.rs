use nalgebra::{Matrix3, Quaternion, Vector3};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

#[derive(Error, Debug)]
pub enum HardwareError {
    #[error("I2C Bus Error: {0}")]
    I2cError(String),
    #[error("SPI Bus Error: {0}")]
    SpiError(String),
    #[error("UART Timeout: {0}")]
    UartTimeout(String),
    #[error("CAN Bus Frame Error: {0}")]
    CanBusError(String),
    #[error("Sensor disconnected: {0}")]
    SensorDisconnected(String),
}

// -----------------------------------------------------------------------------
// IMU (Inertial Measurement Unit) Driver (e.g., MPU9250 / BNO085)
// -----------------------------------------------------------------------------
pub struct ImuState {
    pub accel_mps2: Vector3<f64>,
    pub gyro_rads: Vector3<f64>,
    pub mag_tesla: Vector3<f64>,
    pub temperature_c: f64,
}

pub struct ImuDriver {
    i2c_address: u8,
    calibration_matrix: Matrix3<f64>,
    bias_vector: Vector3<f64>,
}

impl ImuDriver {
    pub fn new(i2c_address: u8) -> Self {
        info!("Initializing IMU on I2C address {:#04x}", i2c_address);
        Self {
            i2c_address,
            calibration_matrix: Matrix3::identity(),
            bias_vector: Vector3::zeros(),
        }
    }

    pub async fn read_raw(&self) -> Result<ImuState, HardwareError> {
        // Simulated I2C transaction
        Ok(ImuState {
            accel_mps2: Vector3::new(0.0, 0.0, 9.81),
            gyro_rads: Vector3::zeros(),
            mag_tesla: Vector3::new(20e-6, 0.0, 45e-6),
            temperature_c: 35.5,
        })
    }

    pub async fn calibrate(&mut self) -> Result<(), HardwareError> {
        info!("Calibrating IMU...");
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        self.bias_vector = Vector3::new(0.01, -0.02, 0.005);
        Ok(())
    }
}

// -----------------------------------------------------------------------------
// GPS / GNSS Driver (e.g., u-blox F9P over UART)
// -----------------------------------------------------------------------------
pub struct GpsState {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude_msl: f64,
    pub satellites_visible: u8,
    pub fix_type: GpsFixType,
    pub hdop: f64,
    pub vdop: f64,
    pub velocity_ned: Vector3<f64>,
}

#[derive(Debug, PartialEq)]
pub enum GpsFixType {
    NoFix,
    Fix2D,
    Fix3D,
    RtkFloat,
    RtkFixed,
}

pub struct GpsDriver {
    uart_port: String,
    baud_rate: u32,
}

impl GpsDriver {
    pub fn new(uart_port: &str, baud_rate: u32) -> Self {
        info!("Initializing GPS on {} at {} baud", uart_port, baud_rate);
        Self {
            uart_port: uart_port.to_string(),
            baud_rate,
        }
    }

    pub async fn poll_nmea(&self) -> Result<GpsState, HardwareError> {
        // Simulated UART read
        Ok(GpsState {
            latitude: 37.7749,
            longitude: -122.4194,
            altitude_msl: 15.0,
            satellites_visible: 12,
            fix_type: GpsFixType::Fix3D,
            hdop: 0.8,
            vdop: 1.2,
            velocity_ned: Vector3::zeros(),
        })
    }
}

// -----------------------------------------------------------------------------
// LiDAR / Depth Sensor Driver (e.g., Ouster or RPLIDAR)
// -----------------------------------------------------------------------------
pub struct PointCloud {
    pub points: Vec<Vector3<f64>>,
    pub timestamp: u64,
}

pub struct LidarDriver {
    ethernet_ip: String,
    rpm: f64,
}

impl LidarDriver {
    pub fn new(ethernet_ip: &str, rpm: f64) -> Self {
        info!("Initializing LiDAR at {} ({} RPM)", ethernet_ip, rpm);
        Self {
            ethernet_ip: ethernet_ip.to_string(),
            rpm,
        }
    }

    pub async fn fetch_scan(&self) -> Result<PointCloud, HardwareError> {
        // Simulated UDP packet stream assembly
        let mut points = Vec::with_capacity(1024);
        for i in 0..1024 {
            let angle = (i as f64) * std::f64::consts::PI * 2.0 / 1024.0;
            points.push(Vector3::new(angle.cos() * 5.0, angle.sin() * 5.0, 0.0));
        }
        Ok(PointCloud {
            points,
            timestamp: 0,
        })
    }
}

// -----------------------------------------------------------------------------
// ESC (Electronic Speed Controller) CAN Bus Driver
// -----------------------------------------------------------------------------
pub struct EscStatus {
    pub rpm: u32,
    pub voltage: f32,
    pub current: f32,
    pub temperature_c: f32,
    pub error_flags: u16,
}

pub struct EscCanDriver {
    can_interface: String,
    num_motors: u8,
}

impl EscCanDriver {
    pub fn new(can_interface: &str, num_motors: u8) -> Self {
        info!(
            "Initializing ESC CAN Driver on {} for {} motors",
            can_interface, num_motors
        );
        Self {
            can_interface: can_interface.to_string(),
            num_motors,
        }
    }

    pub async fn send_pwm_commands(&self, pwms: &[u16]) -> Result<(), HardwareError> {
        if pwms.len() != self.num_motors as usize {
            return Err(HardwareError::CanBusError("PWM array size mismatch".into()));
        }
        // Simulated CAN FD TX
        debug!("Sent PWM commands: {:?}", pwms);
        Ok(())
    }

    pub async fn read_telemetry(&self) -> Result<Vec<EscStatus>, HardwareError> {
        let mut telemetry = Vec::new();
        for _ in 0..self.num_motors {
            telemetry.push(EscStatus {
                rpm: 5000,
                voltage: 22.2,
                current: 1.5,
                temperature_c: 40.0,
                error_flags: 0,
            });
        }
        Ok(telemetry)
    }
}

// -----------------------------------------------------------------------------
// Smart Battery Monitor (SMBus)
// -----------------------------------------------------------------------------
pub struct BatteryState {
    pub voltage_v: f64,
    pub current_a: f64,
    pub remaining_capacity_mah: f64,
    pub state_of_charge_pct: f64,
    pub is_charging: bool,
}

pub struct BatteryMonitor {
    i2c_address: u8,
}

impl BatteryMonitor {
    pub fn new(i2c_address: u8) -> Self {
        Self { i2c_address }
    }

    pub async fn get_state(&self) -> Result<BatteryState, HardwareError> {
        Ok(BatteryState {
            voltage_v: 24.5,
            current_a: -12.5,
            remaining_capacity_mah: 8500.0,
            state_of_charge_pct: 85.0,
            is_charging: false,
        })
    }
}

// -----------------------------------------------------------------------------
// Hardware Abstraction Layer (HAL) Manager
// -----------------------------------------------------------------------------
pub struct EdgeHardwareHAL {
    pub imu: Arc<Mutex<ImuDriver>>,
    pub gps: Arc<Mutex<GpsDriver>>,
    pub lidar: Arc<Mutex<LidarDriver>>,
    pub escs: Arc<Mutex<EscCanDriver>>,
    pub battery: Arc<Mutex<BatteryMonitor>>,
}

impl EdgeHardwareHAL {
    pub async fn init_all() -> Result<Self, HardwareError> {
        info!("Bringing up full Edge Node Hardware Abstraction Layer (HAL)...");

        let mut imu = ImuDriver::new(0x68);
        imu.calibrate().await?;

        Ok(Self {
            imu: Arc::new(Mutex::new(imu)),
            gps: Arc::new(Mutex::new(GpsDriver::new("/dev/ttyS0", 115200))),
            lidar: Arc::new(Mutex::new(LidarDriver::new("192.168.1.200", 600.0))),
            escs: Arc::new(Mutex::new(EscCanDriver::new("can0", 4))),
            battery: Arc::new(Mutex::new(BatteryMonitor::new(0x0B))),
        })
    }

    pub async fn run_diagnostics(&self) -> Result<(), HardwareError> {
        info!("Running comprehensive hardware diagnostics...");
        let bat = self.battery.lock().await.get_state().await?;
        if bat.voltage_v < 18.0 {
            error!(
                "CRITICAL: Battery voltage too low for flight ({}V)",
                bat.voltage_v
            );
        }

        let gps_state = self.gps.lock().await.poll_nmea().await?;
        if gps_state.fix_type == GpsFixType::NoFix {
            warn!("WARNING: No GPS fix detected.");
        }

        info!("Diagnostics passed. Hardware ready.");
        Ok(())
    }
}
