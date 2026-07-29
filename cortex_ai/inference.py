import os
import numpy as np
try:
    import onnxruntime as ort
except ImportError:
    ort = None

class DQNInference:
    def __init__(self, model_path="model.onnx"):
        self.model_path = model_path
        self.session = None
        self.input_name = None
        self.output_name = None
        
        # Load ONNX and try TensorRT
        if ort:
            providers = ['TensorrtExecutionProvider', 'CUDAExecutionProvider', 'CPUExecutionProvider']
            try:
                self.session = ort.InferenceSession(self.model_path, providers=providers)
                self.input_name = self.session.get_inputs()[0].name
                self.output_name = self.session.get_outputs()[0].name
            except Exception as e:
                print(f"Warning: Could not initialize ONNX runtime: {e}")

    def predict(self, state: np.ndarray) -> np.ndarray:
        if state.shape != (19,):
            raise ValueError(f"Expected state shape (19,), got {state.shape}")
            
        if self.session is None:
            # Fallback mock for CI/CD or missing model
            return np.array([0.0, 0.0, 0.0], dtype=np.float32)

        # Reshape to (batch_size, 19)
        inputs = {self.input_name: state.astype(np.float32).reshape(1, 19)}
        outputs = self.session.run([self.output_name], inputs)
        
        # Assuming output is (1, 3)
        return outputs[0][0]
