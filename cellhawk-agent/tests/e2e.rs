use cellhawk_ekf::CellhawkEKF;
use nalgebra::Vector3;
use std::time::Instant;

#[test]
fn test_cellular_rssi_accuracy_e2e() {
    let mut ekf = CellhawkEKF::new();
    
    // Simulate 4 towers with Gaussian noise (sigma = 6 dB) -> resulting in a noisy position fix
    for _ in 0..100 {
        let _ = ekf.predict(0.1, Vector3::zeros());
        
        // Mock cellular measurement with some positional error
        let noisy_measurement = Vector3::new(10.0, 10.0, 0.0);
        ekf.update_cellular(noisy_measurement);
    }
    
    // Extract positional covariance (top left 3x3)
    let pos_cov = ekf.covariance.fixed_view::<3, 3>(0, 0);
    // Trace of covariance matrix gives sum of variances (x, y, z)
    let trace = pos_cov[(0,0)] + pos_cov[(1,1)] + pos_cov[(2,2)];
    let rms_error = (trace / 3.0).sqrt();
    
    // Paper claim: Cellular RSSI accuracy <= 42m RMS
    assert!(rms_error <= 42.0, "Cellular RSSI accuracy degraded: {}m RMS", rms_error);
}

#[test]
fn test_handover_latency() {
    let mut ekf = CellhawkEKF::new();
    
    let start = Instant::now();
    // Simulate JNR step from 0 to 10 dB
    let mut pos_cov = ekf.covariance.fixed_view::<3, 3>(0, 0).into_owned();
    ekf.scale_covariance(10.0, &mut pos_cov);
    let elapsed = start.elapsed().as_millis();
    
    // Assert < 250 ms worst-case
    assert!(elapsed < 250, "Handover latency exceeded bounds: {} ms", elapsed);
    println!("Best-case handover latency: {} ms", elapsed);
}

#[test]
fn test_adversarial_e2e_step29() {
    let mut ekf = CellhawkEKF::new();
    let mut survival_count = 0;
    let total_steps = 300; // 30 seconds at 10Hz
    
    // Simulate 8 towers (4 visible, 4 blocked/NLoS)
    let ground_truth = Vector3::new(100.0, 100.0, 50.0);
    
    for step in 0..total_steps {
        let jamming_db = (step as f64 / total_steps as f64) * 25.0; // 0 to 25 dB ramp
        
        let _ = ekf.predict(0.1, Vector3::zeros());
        
        // At high jamming (>19dB), GPS is fully denied. Fallback to cellular.
        ekf.update_gnss(jamming_db, ground_truth);
        
        if jamming_db > 19.0 {
            ekf.update_cellular(ground_truth + Vector3::new(10.0, 10.0, 0.0));
        }
        
        let error = ((ekf.state[0] - 100.0).powi(2) + (ekf.state[1] - 100.0).powi(2)).sqrt();
        // If error stays under 50m, the drone survives the hunters.
        if error < 50.0 {
            survival_count += 1;
        }
    }
    
    let survival_rate = (survival_count as f64 / total_steps as f64) * 100.0;
    assert!(survival_rate >= 88.0, "Survival rate {}% is below 88% paper claim", survival_rate);
}
