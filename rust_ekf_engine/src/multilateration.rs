use log::{debug, error};
use nalgebra::{DMatrix, DVector, Matrix2, Vector2};

/// Weighted Least Squares Multilateration using Gauss-Newton Optimization
/// Resolves 2D position from N ambient cellular RSSI measurements.
pub struct MultilaterationSolver {
    pub max_iterations: usize,
    pub tolerance: f64,
}

impl MultilaterationSolver {
    pub fn new(max_iterations: usize, tolerance: f64) -> Self {
        Self {
            max_iterations,
            tolerance,
        }
    }

    /// Solves for (x, y) given a set of tower coordinates and estimated distances to each tower.
    /// Uses Weighted Least Squares where weights are proportional to 1 / d^2.
    pub fn solve_2d(
        &self,
        towers: &[(f64, f64)], // Array of (x, y) coordinates of towers
        distances: &[f64],     // Array of estimated distances to each tower
        initial_guess: (f64, f64),
    ) -> Result<(f64, f64), &'static str> {
        let n = towers.len();
        if n < 3 {
            return Err("At least 3 towers required for 2D multilateration");
        }
        if distances.len() != n {
            return Err("Mismatched towers and distances length");
        }

        let mut pos = Vector2::new(initial_guess.0, initial_guess.1);

        for iter in 0..self.max_iterations {
            let mut j = DMatrix::<f64>::zeros(n, 2); // Jacobian
            let mut r = DVector::<f64>::zeros(n); // Residuals
            let mut w = DMatrix::<f64>::zeros(n, n); // Weight matrix

            for i in 0..n {
                let dx = pos[0] - towers[i].0;
                let dy = pos[1] - towers[i].1;
                let current_dist = (dx * dx + dy * dy).sqrt();

                // Residual = calculated_dist - measured_dist
                r[i] = current_dist - distances[i];

                // Jacobian elements (partial derivatives of distance w.r.t x and y)
                if current_dist > 1e-6 {
                    j[(i, 0)] = dx / current_dist;
                    j[(i, 1)] = dy / current_dist;
                } else {
                    j[(i, 0)] = 0.0;
                    j[(i, 1)] = 0.0;
                }

                // Weighting inversely proportional to distance squared
                let dist_sq = distances[i] * distances[i];
                w[(i, i)] = if dist_sq > 1e-6 { 1.0 / dist_sq } else { 1.0 };
            }

            // Normal equations: (J^T * W * J) * delta = -J^T * W * r
            let j_t = j.transpose();
            let j_t_w = &j_t * &w;
            let lhs = &j_t_w * &j; // 2x2 matrix
            let rhs = -(&j_t_w * &r); // 2x1 vector

            // Numerical stability check
            if lhs.iter().any(|val| !val.is_finite()) || rhs.iter().any(|val| !val.is_finite()) {
                error!("NaN or Inf detected in normal equations during multilateration");
                return Err("Numerical instability in Gauss-Newton iteration");
            }

            // Solve for delta using SVD or Inverse
            // We use pseudo-inverse for better stability if near singular
            let lhs_inv = match lhs.clone_owned().pseudo_inverse(1e-9) {
                Ok(inv) => inv,
                Err(_) => return Err("Failed to compute pseudo-inverse (poor geometry)"),
            };

            let delta = lhs_inv * rhs;

            if delta.iter().any(|val| !val.is_finite()) {
                return Err("Numerical divergence in position update");
            }

            // Update position
            pos += delta;

            // Check convergence
            if delta.norm() < self.tolerance {
                debug!("Multilateration converged in {} iterations", iter + 1);
                return Ok((pos[0], pos[1]));
            }
        }

        Err("Multilateration failed to converge")
    }
}

/// Geometric Dilution of Precision (GDOP) Calculation
pub fn calculate_gdop(towers: &[(f64, f64)], estimated_pos: (f64, f64)) -> f64 {
    let n = towers.len();
    if n < 3 {
        return f64::INFINITY;
    }

    let mut h = DMatrix::<f64>::zeros(n, 2);
    for i in 0..n {
        let dx = towers[i].0 - estimated_pos.0;
        let dy = towers[i].1 - estimated_pos.1;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist > 1e-6 {
            h[(i, 0)] = dx / dist;
            h[(i, 1)] = dy / dist;
        }
    }

    let h_t = h.transpose();
    let q = match (h_t * h).try_inverse() {
        Some(inv) => inv,
        None => return f64::INFINITY, // Perfectly collinear towers
    };

    // GDOP is the square root of the trace of the inverse matrix Q
    (q.trace()).sqrt()
}
