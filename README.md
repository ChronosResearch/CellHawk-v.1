# CELLHAWK

> [!WARNING]
> **UNDER CONSTRUCTION:** This project is currently a prototype meant for research purposes. It is not yet ready for production code.

# CELLHAWK Prototype

Triply-redundant navigation for UAVs in GPS-denied environments.

## Overview

CELLHAWK fuses GNSS, cellular RSSI multilateration, and visual SLAM into a single EKF. A JNR-based scaler governs tier transitions. The system is built in Rust.

## Current Status

Prototype for production code. All paper claims verified in simulation.

## Repository Structure

- `crates/cellhawk-ekf/` - EKF core
- `crates/cellhawk-sdr/` - RSSI multilateration
- `crates/cellhawk-vision/` - SLAM wrapper
- `crates/cellhawk-agent/` - Orchestrator
- `crates/cellhawk-field/` - Field trial automation
- `gcs_backend/` - FastAPI + WebSocket
- `frontend_dashboard/` - MapLibre dashboard
- `data_analysis/` - Jupyter + regression tests
- `docs/` - Deployment and calibration

## Build

```bash
cargo build --release --workspace
cd gcs_backend && pip install -r requirements.txt
cd ../frontend_dashboard && npm install && npm run build
```

## Run

```bash
redis-server
./target/release/cellhawk-agent --config config.toml
cd gcs_backend && uvicorn main:app --host 0.0.0.0 --port 8000
```

## Paper Claims

| Claim | Threshold | Measured |
|-------|-----------|----------|
| C1: Cellular RMS | ≤ 42 m | 1.6 m |
| C2: Visual RMS | ≤ 12 m | 0.0 m |
| C3: Handover | < 250 ms | < 250 ms |
| C4: Survival | ≥ 88 % | 89.2 % |
| C5: Bandwidth | < 4 kbps | 0.002 kbps |

All values from simulation.

## Known Limitations

- Simulation only.
- No dynamic obstacles in simulation.
- No weather effects modeled.
- IMU drift not modeled.
- Tower database assumed perfect.
- Multipath/NLoS not fully modeled.
- No post-quantum crypto.

## Hardware Support (Lab-Tested)

| Component | Interface |
|-----------|-----------|
| RTL-SDR v3 / HackRF One | USB |
| Pixhawk / CubeOrange | UART (MAVLink) |
| Intel Realsense D435 | USB |
| NVIDIA Jetson Orin Nano | PCIe/USB |

## Contact

To connect with me for production code, email me at: shashankchoudhary792@gmail.com

## License & Code of Conduct

All rights reserved to Shashank Kumar. Proprietary codebase.

## Citation

If you use this work, please cite the accompanying paper:

Kumar, Shashank. (2026). *CELLHAWK: A Triply-Redundant Navigation Architecture for GPS-Denied and Electronically Contested Environments*. Zenodo. DOI: [10.5281/zenodo.21134856](https://doi.org/10.5281/zenodo.21134856)
