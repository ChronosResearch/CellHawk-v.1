use anyhow::{Context, Result};
use tracing::{info, error};
use std::process::Command;
use std::path::Path;

pub async fn run_analysis() -> Result<()> {
    info!("Invoking Data Analysis Pipeline...");

    let analysis_dir = Path::new("../data_analysis");
    
    // 1. Run Regression Tests
    info!("Running regression_tests.py...");
    let regression_output = Command::new("python3")
        .arg("regression_tests.py")
        .arg("--logs")
        .arg("../logs/")
        .current_dir(analysis_dir)
        .output();

    match regression_output {
        Ok(output) => {
            if output.status.success() {
                info!("Regression tests passed.");
            } else {
                error!("Regression tests failed: {}", String::from_utf8_lossy(&output.stderr));
            }
        },
        Err(e) => {
            error!("Failed to execute python3: {}. Ensure python3 is in PATH.", e);
        }
    }

    // 2. Generate Jupyter Report
    info!("Generating HTML report from Jupyter notebook...");
    let jupyter_output = Command::new("jupyter")
        .args(&["nbconvert", "--to", "html", "--execute", "post_flight_analysis.ipynb"])
        .current_dir(analysis_dir)
        .output();

    match jupyter_output {
        Ok(output) => {
            if output.status.success() {
                info!("HTML Report generated successfully.");
            } else {
                error!("Jupyter nbconvert failed: {}", String::from_utf8_lossy(&output.stderr));
            }
        },
        Err(e) => {
            error!("Failed to execute jupyter: {}. Ensure jupyter is installed.", e);
        }
    }

    info!("Appending final metrics to FIELD_TRIAL_REPORT.md...");
    // Logic to append or update FIELD_TRIAL_REPORT.md would go here.
    
    Ok(())
}
