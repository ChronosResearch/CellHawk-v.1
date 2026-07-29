use pyo3::prelude::*;
use numpy::PyArray1;
use nalgebra::Vector3;
use crate::ekf::EKFNavigationEngine;

#[pyclass]
pub struct CellhawkEKF {
    engine: EKFNavigationEngine,
}

#[pymethods]
impl CellhawkEKF {
    #[new]
    fn new(dt: f64, tier1_threshold: f64, tier2_threshold: f64, steps: u32) -> Self {
        CellhawkEKF {
            engine: EKFNavigationEngine::new(dt, tier1_threshold, tier2_threshold, steps),
        }
    }

    fn predict(&mut self, control_accel: Option<Vec<f64>>) {
        let accel = control_accel.map(|v| Vector3::new(v[0], v[1], v[2]));
        self.engine.predict(accel);
    }

    fn update(
        &mut self,
        jnr_db: f64,
        z_gnss: Option<Vec<f64>>,
        z_cell: Option<Vec<f64>>,
        z_vslam: Option<Vec<f64>>,
    ) -> PyResult<(Vec<f64>, Vec<f64>, f64)> {
        let gnss = z_gnss.map(|v| Vector3::new(v[0], v[1], v[2]));
        let cell = z_cell.map(|v| Vector3::new(v[0], v[1], v[2]));
        let vslam = z_vslam.map(|v| Vector3::new(v[0], v[1], v[2]));

        match self.engine.step(jnr_db, gnss, cell, vslam, None) {
            Ok(state) => {
                Ok((
                    state.position.to_vec(),
                    state.velocity.to_vec(),
                    state.estimated_rms_error_m,
                ))
            }
            Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(format!("EKF Error: {:?}", e))),
        }
    }
}

#[pymodule]
fn cellhawk_ekf(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<CellhawkEKF>()?;
    Ok(())
}
