import sys
import os

# Add the directory containing the compiled .pyd file to the python path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'rust_ekf_engine', 'target', 'release')))

try:
    import cellhawk_ekf
    print("SUCCESS: Imported cellhawk_ekf native module!")
except ImportError as e:
    print(f"FAIL: Could not import cellhawk_ekf: {e}")
    sys.exit(1)

def main():
    print("Initializing Rust EKF via PyO3...")
    ekf = cellhawk_ekf.CellhawkEKF(0.1, 10.0, 20.0, 5)
    
    print("Running Predict Step...")
    ekf.predict([0.1, 0.0, -9.81])
    
    print("Running Update Step with GNSS data...")
    try:
        # jnr_db, gnss, cell, vslam
        pos, vel, rms = ekf.update(12.0, [100.0, 200.0, 50.0], None, None)
        print(f"State -> Pos: {pos}, Vel: {vel}, RMS Error: {rms:.2f}")
        print("VERDICT: PASS. Zero-copy / Python interop successful.")
    except Exception as e:
        print(f"VERDICT: FAIL. Exception during update: {e}")

if __name__ == "__main__":
    main()
