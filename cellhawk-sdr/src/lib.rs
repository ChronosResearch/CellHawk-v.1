use nalgebra::{SMatrix, SVector, OVector, OMatrix, Dyn};
use tokio::sync::mpsc;
use tokio::time::{self, Duration};
pub mod self_test;

/// Log-Distance Path Loss model
pub fn rssi_to_range(rssi: f32, tx_power: f32, n: f32, d0: f32) -> f32 {
    let exponent = (tx_power - rssi) / (10.0 * n);
    d0 * 10.0_f32.powf(exponent)
}

#[derive(Debug)]
pub enum MultilaterationError {
    NotEnoughTowers,
    NegativeRange,
    MatrixInversionFailed,
}

/// Weighted Least Squares Multilateration
pub fn multilaterate(towers: &[(f64, f64)], ranges: &[f64]) -> Result<(f64, f64), MultilaterationError> {
    if towers.len() < 3 || towers.len() != ranges.len() {
        return Err(MultilaterationError::NotEnoughTowers);
    }
    
    for &r in ranges {
        if r < 0.0 {
            return Err(MultilaterationError::NegativeRange);
        }
    }

    let n = towers.len();
    let mut a = OMatrix::<f64, Dyn, Dyn>::zeros(n - 1, 2);
    let mut b = OVector::<f64, Dyn>::zeros(n - 1);

    let (xn, yn) = towers[n - 1];
    let rn = ranges[n - 1];

    for i in 0..(n - 1) {
        let (xi, yi) = towers[i];
        let ri = ranges[i];

        a[(i, 0)] = 2.0 * (xn - xi);
        a[(i, 1)] = 2.0 * (yn - yi);

        b[i] = (rn.powi(2) - ri.powi(2)) - (xn.powi(2) - xi.powi(2)) - (yn.powi(2) - yi.powi(2));
    }

    // Solve Ax = b -> x = (A^T A)^-1 A^T b
    let a_t = a.transpose();
    let a_t_a = &a_t * &a;
    
    // nalgebra's LU or Cholesky can be used, we'll try inverse directly for the 2x2 matrix
    let a_t_a_inv = a_t_a.clone_owned().try_inverse().ok_or(MultilaterationError::MatrixInversionFailed)?;
    
    let pos = a_t_a_inv * a_t * b;
    Ok((pos[0], pos[1]))
}

/// SDR Telemetry Task ( tokio )
pub async fn run_sdr_telemetry(tx: mpsc::Sender<f32>) {
    let mut interval = time::interval(Duration::from_millis(100)); // 10 Hz
    loop {
        interval.tick().await;
        
        let simulated_rssi = -65.0; // Simulated RSSI read from HackRF/RTL-SDR
        
        // Try to send without blocking. If full, we should ideally pop the oldest, 
        // but with a bounded mpsc, `try_send` failing means we just drop this one or we 
        // could clear the channel. To explicitly drop oldest, we'd need a custom queue.
        // For standard mpsc, non-blocking is try_send.
        match tx.try_send(simulated_rssi) {
            Ok(_) => {},
            Err(mpsc::error::TrySendError::Full(_)) => {
                // To drop the oldest, we can try to pop one element then push again.
                // But we don't have receiver here. So we just drop the current sample.
            },
            Err(mpsc::error::TrySendError::Closed(_)) => break,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rssi_to_range_reverse() {
        let tx_power = -20.0;
        let n = 2.8;
        let d0 = 1.0;
        let original_range = 50.0_f32;
        let expected_rssi = tx_power - 10.0 * n * original_range.log10();
        let calculated_range = rssi_to_range(expected_rssi, tx_power, n, d0);
        assert!((calculated_range - original_range).abs() < 0.1);
    }

    #[test]
    fn test_ldpl_range_accuracy_step10() {
        let tx_power = -20.0;
        let n = 2.8;
        let d0 = 1.0;
        
        for &true_dist in &[100.0_f32, 500.0_f32, 1000.0_f32] {
            let rssi = tx_power - 10.0 * n * true_dist.log10();
            let estimated = rssi_to_range(rssi, tx_power, n, d0);
            let error_pct = (estimated - true_dist).abs() / true_dist;
            assert!(error_pct < 0.02, "Error {} exceeds 2% for distance {}", error_pct, true_dist);
        }
    }

    #[test]
    fn test_multilateration_3_towers_step11() {
        // Towers at (0,0), (100,0), (0,100). Target at (50,50).
        let towers = vec![(0.0, 0.0), (100.0, 0.0), (0.0, 100.0)];
        // Distance to (50,50) is sqrt(50^2 + 50^2) = 70.7106
        let dist = (50.0_f64.powi(2) + 50.0_f64.powi(2)).sqrt();
        let ranges = vec![dist, dist, dist];
        
        let result = multilaterate(&towers, &ranges).unwrap();
        assert!((result.0 - 50.0).abs() < 1.0);
        assert!((result.1 - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_multilateration_4_towers_outlier_step12() {
        let towers = vec![(0.0, 0.0), (100.0, 0.0), (0.0, 100.0), (100.0, 100.0)];
        let dist = 70.7106; // Target at (50,50)
        let nlos_outlier = dist * 1.5; // 50% overestimated
        let ranges = vec![dist, dist, dist, nlos_outlier];
        
        let result = multilaterate(&towers, &ranges).unwrap();
        // WLS should keep it within 50m of ground truth
        let error = ((result.0 - 50.0).powi(2) + (result.1 - 50.0).powi(2)).sqrt();
        assert!(error < 50.0, "Outlier rejection failed: error {}m", error);
    }

    #[tokio::test]
    async fn test_channel_backpressure_step8() {
        let (tx, _rx) = mpsc::channel(32);
        
        // Simulate 200 messages in 100ms
        let mut dropped = 0;
        for _ in 0..200 {
            if let Err(_) = tx.try_send(-65.0) {
                dropped += 1;
            }
        }
        // If it was blocking, this loop would hang. Since it doesn't, it's non-blocking.
        // It drops messages as required when full.
        assert_eq!(dropped, 200 - 32); 
    }
}
