# CELLHAWK Troubleshooting Guide

## Common Issues

### 1. Agent Fails to Start (Bind Error)
**Symptom**: `ZMQ bind failed: Address already in use`
**Cause**: Another instance of the agent or a stalled ZMQ socket is occupying port 5555.
**Resolution**:
```bash
sudo lsof -i :5555
kill -9 <PID>
```

### 2. Drone Rejects MAVLink Commands
**Symptom**: Agent logs `Sending ARM command...` but drone remains disarmed.
**Cause**: PX4/ArduPilot requires specific pre-flight checks to pass before accepting offboard mode.
**Resolution**:
- Check GPS fix in QGroundControl (if operating in Tier 1).
- Ensure the hardware switch (Dead Man's Switch) is engaged.
- Verify `SYS_COMPANION` parameter is set correctly on the flight controller.

### 3. SDR Telemetry Dropping Frames
**Symptom**: Log warning `WARNING: SDR channel full, dropping frame.`
**Cause**: The orchestrator loop is blocked (>100ms) or the HackRF is producing samples faster than the CPU can process them.
**Resolution**:
- Verify the orchestrator is running on an isolated CPU core (`taskset`).
- Check system load. If thermal throttling is occurring on the Jetson, increase fan speed.

### 4. Vision SLAM Drift
**Symptom**: `Visual SLAM update exceeded noise bounds: > 12m` in tests, or drone drifting physically.
**Cause**: Feature-poor environment (e.g. blank white walls) or rapid lighting changes.
**Resolution**:
- If testing indoors, place ArUco markers or posters to increase feature points.
- Ensure the Realsense camera lens is clean.

### 5. Redis Danger Grid Connection Failed
**Symptom**: `RedisError: Connection Refused`
**Cause**: The local or remote Redis server is down.
**Resolution**:
- Start the redis server: `systemctl start redis-server`
- Check `config.toml` to ensure the correct Redis URI is specified.
