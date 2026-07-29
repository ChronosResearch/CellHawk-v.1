use rand::thread_rng;
use rand_distr::{Distribution, Normal};

/// Log-Distance Path Loss (LDPL) Model for converting RSSI to Range
pub struct LogDistancePathLoss {
    pub reference_power_dbm: f64,  // P_t at d_0
    pub reference_distance_m: f64, // d_0
    pub path_loss_exponent: f64,   // n
}

impl LogDistancePathLoss {
    pub fn new(
        reference_power_dbm: f64,
        reference_distance_m: f64,
        path_loss_exponent: f64,
    ) -> Self {
        Self {
            reference_power_dbm,
            reference_distance_m,
            path_loss_exponent,
        }
    }

    /// Converts measured RSSI (dBm) to estimated distance (meters)
    /// d = d_0 * 10 ^ ((P_t - RSSI) / (10 * n))
    pub fn rssi_to_distance(&self, rssi_dbm: f64) -> f64 {
        let exponent = (self.reference_power_dbm - rssi_dbm) / (10.0 * self.path_loss_exponent);
        self.reference_distance_m * 10.0_f64.powf(exponent)
    }

    /// Converts known distance (meters) to expected RSSI (dBm) with optional log-normal shadowing
    pub fn distance_to_rssi(&self, distance_m: f64, shadowing_sigma_db: f64) -> f64 {
        if distance_m <= 0.0 {
            return self.reference_power_dbm;
        }

        let mut rssi = self.reference_power_dbm
            - 10.0 * self.path_loss_exponent * (distance_m / self.reference_distance_m).log10();

        if shadowing_sigma_db > 0.0 {
            let normal = Normal::new(0.0, shadowing_sigma_db).unwrap();
            let mut rng = thread_rng();
            rssi += normal.sample(&mut rng); // Add X_sigma ~ N(0, sigma^2)
        }

        rssi
    }
}

/// Rician Fading Channel Model for NLoS / Multipath Simulation
pub struct RicianFading {
    pub k_factor_db: f64,
}

impl RicianFading {
    pub fn new(k_factor_db: f64) -> Self {
        Self { k_factor_db }
    }

    /// Applies Rician fading to a signal amplitude (linear scale).
    /// Returns the faded amplitude.
    pub fn apply_fading(&self, direct_amplitude: f64) -> f64 {
        let k_linear = 10.0_f64.powf(self.k_factor_db / 10.0);

        // Power distribution: P_direct + P_scattered = 1.0
        // K = P_direct / P_scattered
        // P_scattered = 1 / (K + 1)
        // P_direct = K / (K + 1)

        let p_total = direct_amplitude * direct_amplitude;
        let p_scattered = p_total / (k_linear + 1.0);
        let p_direct = p_total * k_linear / (k_linear + 1.0);

        let direct_amp = p_direct.sqrt();
        let scattered_sigma = (p_scattered / 2.0).sqrt(); // For I and Q components

        let normal = Normal::new(0.0, scattered_sigma).unwrap();
        let mut rng = thread_rng();

        // Rician distribution components
        let x = direct_amp + normal.sample(&mut rng);
        let y = normal.sample(&mut rng);

        (x * x + y * y).sqrt()
    }
}
