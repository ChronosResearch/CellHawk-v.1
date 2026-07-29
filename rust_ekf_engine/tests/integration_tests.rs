use cellhawk_ekf::ekf::EKFNavigationEngine;
use nalgebra::Vector3;
use std::time::Instant;

#[test]
fn test_tier_handover_latency() {
    let mut engine = EKFNavigationEngine::new(0.1, 10.0, 20.0, 5);
    
    // Simulate GNSS failure (JNR spikes)
    let start = Instant::now();
    engine.step(15.0, None, Some(Vector3::new(10.0, 10.0, 0.0)), None, None).unwrap();
    let elapsed = start.elapsed();
    
    // Should be < 150ms
    assert!(elapsed.as_millis() < 150, "Handover took too long: {} ms", elapsed.as_millis());
}

#[test]
fn test_cellular_rssi_accuracy() {
    let mut engine = EKFNavigationEngine::new(0.1, 10.0, 20.0, 5);
    
    // Simulated noisy RSSI resulting in an RMS error bound
    let state = engine.step(15.0, None, Some(Vector3::new(10.0, 10.0, 0.0)), None, None).unwrap();
    
    // Assert error <= 42m RMS
    assert!(state.estimated_rms_error_m <= 42.0, "RSSI Accuracy degraded: {}", state.estimated_rms_error_m);
}

#[test]
fn test_visual_slam_accuracy() {
    let mut engine = EKFNavigationEngine::new(0.1, 10.0, 20.0, 5);
    
    let state = engine.step(25.0, None, None, Some(Vector3::new(10.0, 10.0, 0.0)), None).unwrap();
    
    // Assert error ~ 12m RMS (allow 20% regression -> up to 14.4)
    assert!(state.estimated_rms_error_m <= 14.4, "Visual SLAM degraded: {}", state.estimated_rms_error_m);
}
