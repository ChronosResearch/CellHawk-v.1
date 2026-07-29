use rtlsdr::{RTLSDRDevice, RTLSDRError};
use tracing::{info, error};

pub trait RssiSource {
    fn get_rssi(&mut self, frequency_hz: u32) -> Result<f32, String>;
}

pub struct RtlSdrWrapper {
    device: RTLSDRDevice,
}

impl RtlSdrWrapper {
    pub fn new(device_index: i32) -> Result<Self, RTLSDRError> {
        let mut device = rtlsdr::open(device_index)?;
        device.set_sample_rate(2_048_000)?;
        device.set_tuner_gain_mode(true)?;
        device.set_tuner_gain(400)?; // 40.0 dB
        
        info!("RTL-SDR initialized successfully.");
        Ok(Self { device })
    }
}

impl RssiSource for RtlSdrWrapper {
    fn get_rssi(&mut self, frequency_hz: u32) -> Result<f32, String> {
        self.device.set_center_freq(frequency_hz).map_err(|e| format!("{:?}", e))?;
        
        // Read 16k samples synchronously
        let mut buffer = vec![0u8; 16384];
        self.device.read_sync(&mut buffer).map_err(|e| format!("{:?}", e))?;
        
        // Compute I/Q magnitude (RSSI approximation)
        let mut power_sum = 0.0;
        for i in (0..buffer.len()).step_by(2) {
            let i_val = (buffer[i] as f32) - 127.5;
            let q_val = (buffer[i+1] as f32) - 127.5;
            power_sum += i_val * i_val + q_val * q_val;
        }
        
        let avg_power = power_sum / (buffer.len() as f32 / 2.0);
        let rssi_dbm = 10.0 * avg_power.log10();
        
        Ok(rssi_dbm)
    }
}
