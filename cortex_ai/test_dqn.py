import numpy as np
from inference import DQNInference

def test_dqn_output_bounds():
    dqn = DQNInference()
    # Pass random state
    state = np.random.randn(19).astype(np.float32)
    output = dqn.predict(state)
    
    if output.shape != (3,):
        raise ValueError(f"Output shape {output.shape} != (3,)")
    
    # Check bounds (heading -180..180, climb -5..5)
    # Output mock currently returns 0.0, 0.0, 0.0
    heading = output[0]
    climb = output[1]
    
    if not (-180.0 <= heading <= 180.0):
        raise ValueError(f"Heading out of bounds: {heading}")
    if not (-5.0 <= climb <= 5.0):
        raise ValueError(f"Climb out of bounds: {climb}")
    print("PASS: DQN output bounds verified.")

if __name__ == "__main__":
    test_dqn_output_bounds()
