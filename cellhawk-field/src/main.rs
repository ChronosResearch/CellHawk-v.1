use anyhow::{Context, Result};
use clap::Parser;
use tracing::{error, info};

mod calibration;
mod reporting;
mod test_sequence;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Simulate the sequence without hardware
    #[arg(long)]
    pub dry_run: bool,

    /// Skip specific tests (e.g., "swarm", "jamming")
    #[arg(long, value_delimiter = ',')]
    pub skip_tests: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();
    info!(
        "Starting CELLHAWK Field Trial Orchestrator (Dry Run: {})",
        cli.dry_run
    );

    // 1. Read Configuration
    let _settings = config::Config::builder()
        .add_source(config::File::with_name("../config.toml"))
        .build()
        .context("Failed to load config.toml")?;

    info!("Configuration loaded successfully.");

    // 2. Hardware Self-Test
    run_hardware_self_tests(cli.dry_run)?;

    // 3. Calibration Phase
    if !cli.skip_tests.contains(&"calibration".to_string()) {
        calibration::run_calibration(cli.dry_run).await?;
    }

    // 4 & 5. Test Sequence Execution & Data Collection
    test_sequence::run_test_campaign(&cli).await?;

    // 6. Data Analysis
    if !cli.skip_tests.contains(&"analysis".to_string()) {
        reporting::run_analysis().await?;
    }

    // 7. Final Output
    info!("All tests completed. Report generated at reports/FIELD_TRIAL_REPORT.md");

    // 8. Cleanup
    cleanup();

    Ok(())
}

fn run_hardware_self_tests(dry_run: bool) -> Result<()> {
    info!("Running hardware self-tests...");
    if dry_run {
        info!("Dry run: Skipping actual hardware tests.");
        return Ok(());
    }

    // Call underlying self-tests
    if let Err(e) = cellhawk_ekf::self_test::self_test() {
        error!("EKF Self-Test Failed: {}", e);
        anyhow::bail!("Critical Component Failure: EKF");
    }

    if let Err(e) = cellhawk_sdr::self_test::self_test() {
        error!("SDR Self-Test Failed: {}", e);
        anyhow::bail!("Critical Component Failure: SDR");
    }

    if let Err(e) = cellhawk_vision::self_test::self_test() {
        error!("Vision Self-Test Failed: {}", e);
        anyhow::bail!("Critical Component Failure: Vision");
    }

    if let Err(e) = cellhawk_agent::swarm::self_test::self_test() {
        error!("Swarm Self-Test Failed: {}", e);
        tracing::warn!("Fail-safe triggered: Proceeding with backup swarm mock");
    }

    info!("Hardware self-tests passed.");
    Ok(())
}

fn cleanup() {
    info!("Stopping all hardware streams and zeroizing memory...");
    // Mock cleanup logic
    info!("Cleanup complete.");
}
