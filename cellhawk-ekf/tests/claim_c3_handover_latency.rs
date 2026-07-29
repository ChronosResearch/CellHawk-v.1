use cellhawk_ekf::CellhawkEKF;
use nalgebra::SVector;
use std::time::Instant;

#[test]
fn claim_c3_handover_latency() {
    let mut ekf = CellhawkEKF::new();
    let start = Instant::now();
    
    ekf.scale_covariance(10.0);
    
    let elapsed = start.elapsed().as_millis();
    println!("Handover latency: {} ms", elapsed);
    assert!(elapsed < 250);
}
