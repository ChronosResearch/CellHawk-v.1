#![no_main]
use libfuzzer_sys::fuzz_target;
use cellhawk_ekf::ekf::EKFNavigationEngine;
use nalgebra::Vector3;

fuzz_target!(|data: (f64, [f64; 3])| {
    let (jnr, gnss_arr) = data;
    
    // Fuzzing extreme values (NaN, Inf, negative RSSI)
    let mut engine = EKFNavigationEngine::new(0.1, 10.0, 20.0, 5);
    
    let gnss = Vector3::new(gnss_arr[0], gnss_arr[1], gnss_arr[2]);
    
    // Ensure it doesn't panic on malformed inputs or infinite covariance
    let _ = engine.step(jnr, Some(gnss), None, None, None);
});
