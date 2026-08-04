# CELLHAWK Security & Threat Model

This document outlines the security architecture of the CELLHAWK system, focusing on its resilience against GPS-denial, electronic warfare, and cyber threats.

## 1. Threat Model

### Adversarial Capabilities
- **GPS Jamming & Spoofing**: Attackers can deny GNSS signals or inject false coordinates.
- **Cellular Network Spoofing (Stingrays)**: Attackers may deploy fake base stations.
- **Kinetic Interception**: Hunter drones or ground-based countermeasures.
- **C2 Link Hijacking**: Attempts to intercept or modify telemetry and commands.

### System Assumptions
- The hardware root of trust (Jetson Secure Boot) is uncompromised.
- The Redis Danger Grid operates within a trusted LAN/VLAN.
- Local physical access to the drone is protected post-deployment.

## 2. Security Guarantees

### Triply-Redundant Navigation
1. **Tier 1 (Optimal)**: GNSS + EKF. JNR is < 6 dB.
2. **Tier 2 (Degraded)**: GNSS denied. System smoothly interpolates to Cellular Multilateration (WLS) + IMU over 5 cycles (< 250ms latency).
3. **Tier 3 (Severely Denied)**: Cellular denied. Relies entirely on Visual SLAM + IMU.

### Cryptographic Protections
- **C2 Links**: All MAVLink and C2 telemetry is encrypted using mTLS via Rustls.
- **Data Serialization**: Protobuf serialization enforces strict size limits to prevent buffer overflow/injection attacks.

## 3. Data Privacy & Leakage
- **No Hardcoded Secrets**: All keys, passwords, and tokens are injected via environment variables at runtime.
- **Log Redaction**: Sensitive variables (GNSS coordinates, raw RSSI, tower IDs) are omitted from standard `info!` and `debug!` logs.
- **Zero-Trust**: Subcomponents treat incoming measurements with suspicion (e.g., EKF Huber loss down-weights outliers > 3σ).

## 4. Audit Log
Any anomalous tier transitions, watchdog timeouts, or communication failures are permanently appended to `/var/log/cellhawk_audit.log` for post-mission forensics.
