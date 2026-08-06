use cellhawk_ekf::CellhawkEKF;
use nalgebra::SVector;

#[test]
fn claim_c4_survival_rate() {
    let mut ekf = CellhawkEKF::new();
    let mut survival_count = 0;
    let total_steps = 100;
    let ground_truth = SVector::<f64, 3>::new(100.0, 100.0, 50.0);

    for _step in 0..total_steps {
        let jamming_db = 25.0; // 25dB jamming
        let _ = ekf.predict(0.1, SVector::zeros());
        ekf.scale_covariance(jamming_db);

        ekf.update_cellular(ground_truth + SVector::<f64, 3>::new(10.0, 10.0, 0.0));

        let error = ((ekf.state[0] - 100.0).powi(2) + (ekf.state[1] - 100.0).powi(2)).sqrt();
        if error < 50.0 {
            survival_count += 1;
        }
    }

    let survival_rate = (survival_count as f64 / total_steps as f64) * 100.0;
    println!("Survival Rate: {}", survival_rate);
    assert!(survival_rate >= 88.0);
}
