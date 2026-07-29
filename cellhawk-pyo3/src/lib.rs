use pyo3::prelude::*;
use pyo3::types::{PyModule, PyTuple};
use numpy::{IntoPyArray, PyArray1, PyReadonlyArray1};

pub fn call_dqn(state: [f32; 19]) -> Result<[f32; 3], PyErr> {
    Python::with_gil(|py| {
        // Import sys and add cortex_ai to path
        let sys = py.import_bound("sys")?;
        let path = sys.getattr("path")?;
        let path_list = path.downcast::<pyo3::types::PyList>()?;
        // Append absolute or relative path to cortex_ai
        path_list.insert(0, "./cortex_ai")?;

        let inference_mod = py.import_bound("inference")?;
        let dqn_class = inference_mod.getattr("DQNInference")?;
        // Mock init without a real model file
        let dqn_instance = dqn_class.call0()?;

        // Convert input array
        let state_py = state.into_pyarray_bound(py);

        // Call predict
        let args = PyTuple::new_bound(py, &[state_py.as_any()]);
        let result = dqn_instance.call_method1("predict", args)?;
        
        let out_arr = result.extract::<PyReadonlyArray1<f32>>()?;
        let slice = out_arr.as_slice()?;
        
        if slice.len() == 3 {
            Ok([slice[0], slice[1], slice[2]])
        } else {
            Err(pyo3::exceptions::PyValueError::new_err("Output shape mismatch"))
        }
    })
}
