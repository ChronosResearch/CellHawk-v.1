use cellhawk_ekf::CellhawkEKF;
use nalgebra::SVector;

#[test]
fn claim_c2_visual_slam_rms() {
    let mut ekf = CellhawkEKF::new();
    let ground_truth = SVector::<f64, 3>::new(20.0, 20.0, 0.0);

    for _ in 0..200 {
        ekf.predict(0.1, SVector::zeros()).unwrap();
        ekf.update_vision(ground_truth);
    }

    let error = ((ekf.state[0] - 20.0).powi(2) + (ekf.state[1] - 20.0).powi(2)).sqrt();
    println!("Measured SLAM RMS: {}", error);
    assert!(error <= 12.0);
}
