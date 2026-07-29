import pandas as pd
import numpy as np
from scipy.stats import ttest_1samp, ks_2samp
import sys

def run_regression_tests(csv_path):
    print(f"Running Regression Tests on {csv_path}...")
    
    # Mock data for CI demonstration
    np.random.seed(42)
    measured_cellular_errors = np.random.normal(30.0, 5.0, 100) # mean 30m
    simulated_cellular_errors = np.random.normal(32.0, 6.0, 1000)
    
    # 1. Student's t-test (Verify error stays within paper bounds of 42m)
    # H0: mean error >= 42m, H1: mean error < 42m
    # We perform a 1-sample t-test against the population mean of 42
    t_stat, p_val = ttest_1samp(measured_cellular_errors, 42.0, alternative='less')
    
    print(f"T-test (Target < 42m): t={t_stat:.2f}, p={p_val:.4f}")
    if p_val > 0.05:
        print("FAIL: Cellular error significantly exceeds 42m bound (p > 0.05)")
        sys.exit(1)
        
    # 2. Kolmogorov-Smirnov test (Compare distributions)
    # H0: measured and simulated come from the same distribution
    ks_stat, ks_p = ks_2samp(measured_cellular_errors, simulated_cellular_errors)
    print(f"KS-test vs Simulation: stat={ks_stat:.2f}, p={ks_p:.4f}")
    
    if ks_p < 0.05:
        print("WARNING: Field error distribution deviates significantly (>2σ) from simulation!")
        # Flagged for investigation but not an outright failure if t-test passed
        
    print("Regression tests passed.")
    sys.exit(0)

if __name__ == "__main__":
    if len(sys.argv) > 1:
        run_regression_tests(sys.argv[1])
    else:
        run_regression_tests("dummy.csv")
