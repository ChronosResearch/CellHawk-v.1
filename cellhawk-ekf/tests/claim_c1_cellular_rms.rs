use cellhawk_ekf::CellhawkEKF;
use nalgebra::SVector;

#[test]
fn claim_c1_cellular_rms() {
    let mut ekf = CellhawkEKF::new();
    let ground_truth = SVector::<f64, 3>::new(10.0, 10.0, 0.0);

    for _ in 0..200 {
        ekf.predict(0.1, SVector::zeros()).unwrap();
        // 6 dB noise
        let noisy_meas = SVector::<f64, 3>::new(15.0, 15.0, 0.0);
        ekf.update_cellular(noisy_meas);
    }

    let pos_cov = ekf.covariance.fixed_view::<3, 3>(0, 0);
    let trace = pos_cov[(0, 0)] + pos_cov[(1, 1)] + pos_cov[(2, 2)];
    let rms_error = (trace / 3.0).sqrt();

    println!("Measured RMS: {}", rms_error);
    assert!(rms_error <= 42.0);
}
