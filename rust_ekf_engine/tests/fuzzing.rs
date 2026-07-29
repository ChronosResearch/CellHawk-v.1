use cellhawk_core::ekf::EKFNavigationEngine;
use cellhawk_core::types::NavigationTier;
use nalgebra::Vector3;
use rand::Rng;

/// Massive Industrial Verification Suite
/// In a true production environment, these property-based tests run for hours
/// to ensure the flight controller never diverges under extreme stochastic noise.

#[test]
fn monte_carlo_fuzz_ekf_divergence() {
    let mut rng = rand::thread_rng();

    // Fuzz the EKF over 10,000 extreme randomized iterations
    for _ in 0..10_000 {
        let mut ekf = EKFNavigationEngine::new(0.01, 6.0, 19.0, 5);
        ekf.set_initial_state(Vector3::zeros(), None);

        let jnr = rng.gen_range(-10.0..50.0);

        // Randomly drop out sensors to simulate hardware failures
        let gnss = if rng.gen_bool(0.7) {
            Some(Vector3::new(
                rng.gen_range(-100.0..100.0),
                rng.gen_range(-100.0..100.0),
                rng.gen_range(-10.0..10.0),
            ))
        } else {
            None
        };

        let cell = if rng.gen_bool(0.5) {
            Some(Vector3::new(
                rng.gen_range(-100.0..100.0),
                rng.gen_range(-100.0..100.0),
                0.0,
            ))
        } else {
            None
        };

        // Inject extreme acceleration impulses (e.g., wind gusts or explosions)
        let accel = if rng.gen_bool(0.1) {
            Some(Vector3::new(
                rng.gen_range(-50.0..50.0),
                rng.gen_range(-50.0..50.0),
                rng.gen_range(-50.0..50.0),
            ))
        } else {
            Some(Vector3::zeros())
        };

        let state = ekf.step(jnr, gnss, cell, None, accel);

        // Property 1: State must NEVER contain NaN or Infinity
        assert!(
            state.position[0].is_finite(),
            "EKF diverged to NaN on position X"
        );
        assert!(
            state.position[1].is_finite(),
            "EKF diverged to NaN on position Y"
        );
        assert!(
            state.position[2].is_finite(),
            "EKF diverged to NaN on position Z"
        );

        // Property 2: Covariance must remain positive semi-definite (approximated by checking diagonals)
        assert!(
            state.covariance[(0, 0)] >= 0.0,
            "Negative variance detected"
        );
        assert!(
            state.covariance[(1, 1)] >= 0.0,
            "Negative variance detected"
        );
        assert!(
            state.covariance[(2, 2)] >= 0.0,
            "Negative variance detected"
        );
    }
}
