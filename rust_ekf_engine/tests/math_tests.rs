use cellhawk_core::multilateration::{calculate_gdop, MultilaterationSolver};
use cellhawk_core::rf_model::{LogDistancePathLoss, RicianFading};
use nalgebra::Vector3;

#[test]
fn test_rf_path_loss() {
    // Reference at 1m is -40 dBm, Path loss exponent 2.8
    let ldpl = LogDistancePathLoss::new(-40.0, 1.0, 2.8);

    // Distance should be 1000m
    let expected_dist = 1000.0;
    let expected_rssi = -40.0 - 10.0 * 2.8 * (1000.0_f64).log10();

    let calculated_rssi = ldpl.distance_to_rssi(expected_dist, 0.0);
    assert!((calculated_rssi - expected_rssi).abs() < 1e-4);

    let calculated_dist = ldpl.rssi_to_distance(calculated_rssi);
    assert!((calculated_dist - expected_dist).abs() < 1e-4);
}

#[test]
fn test_multilateration_2d() {
    let solver = MultilaterationSolver::new(100, 1e-6);

    // Four towers placed around the drone
    let towers = vec![(0.0, 1000.0), (1000.0, 0.0), (0.0, -1000.0), (-1000.0, 0.0)];

    // Drone is at (100.0, 200.0)
    let true_pos = (100.0, 200.0);

    let mut distances = Vec::new();
    for t in &towers {
        let dx = t.0 - true_pos.0;
        let dy = t.1 - true_pos.1;
        distances.push((dx * dx + dy * dy).sqrt());
    }

    // Initial guess at origin
    let initial_guess = (0.0, 0.0);

    let result = solver.solve_2d(&towers, &distances, initial_guess);
    assert!(result.is_ok());

    let est_pos = result.unwrap();
    assert!((est_pos.0 - true_pos.0).abs() < 1e-3);
    assert!((est_pos.1 - true_pos.1).abs() < 1e-3);
}

#[test]
fn test_gdop() {
    let towers = vec![
        (100.0, 100.0),
        (100.0, -100.0),
        (-100.0, 100.0),
        (-100.0, -100.0),
    ];
    let pos = (0.0, 0.0);
    let gdop = calculate_gdop(&towers, pos);
    // Symmetric 4 towers around origin gives GDOP of sqrt(2) ~ 1.414
    assert!(gdop < 1.5);
}
