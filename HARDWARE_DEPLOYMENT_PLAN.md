# CELLHAWK Hardware Deployment Plan

## 1. Component Selection
- **Compute:** NVIDIA Jetson Orin Nano (8GB RAM) - optimal for running ORB-SLAM2, PyO3 Python bindings, and the Rust EKF.
- **SDR:** RTL-SDR (0.5W) - chosen for weight reduction and power-budget constraints over the HackRF One, given the 2km operating radius.
- **Flight Controller:** Pixhawk CubeOrange - industry standard, robust MAVLink support.
- **Antenna:** Taoglas TG.09 - high gain cellular bands.

## 2. Integration & Connections
- **Power Budget:** 
  - Jetson Orin Nano: ~15W (powered via 5V UBEC from 6S LiPo).
  - RTL-SDR: ~2.5W (powered via Jetson USB 3.0).
  - Pixhawk: ~3W (powered via dedicated Power Brick).
- **Wiring:**
  - RTL-SDR -> USB 3.0 -> Jetson Orin Nano.
  - Jetson Orin Nano (UART2) -> Pixhawk (TELEM2) for MAVLink telemetry and control setpoints.

## 3. Static Ground Tests (Phase 2)

### Tower Database Validation
1. Power up the SDR on the bench.
2. Cross-reference scanned PLMN and Cell IDs with downloaded OpenCellID database (`towers.json`).
3. Verify resolution of at least 4 active towers with an SNR > 5dB.
4. Log offset errors against the bench RTK-GPS reference (target < 5m static offset).

### Full System Integration Test (Tethered)
1. Provide a dummy GNSS signal via a HackRF SDR spoofing rig (controlled, indoor faraday cage or coax direct).
2. Start the CELLHAWK agent.
3. Validate heartbeat MAVLink messages are received by the GCS.
4. Verify all internal Auto-QA watchdogs (`ekf`, `sdr`, `vision`, `swarm`) remain healthy for >30 minutes.
