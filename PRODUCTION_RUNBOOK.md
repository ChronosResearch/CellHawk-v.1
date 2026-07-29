# CELLHAWK Production Runbook

## 1. Startup Commands
Start the required background services first:
```bash
redis-server --daemonize yes
```
Start the FastAPI Ground Control Station (GCS):
```bash
uvicorn gcs_backend.main:app --host 0.0.0.0 --port 8000
```
Start the CELLHAWK Rust Orchestrator:
```bash
RUST_LOG=cellhawk=info ./target/release/cellhawk-agent
```

## 2. Required Environment Variables
Ensure the following variables are injected via Kubernetes Secrets or `.env` files:
- `CELLHAWK_DRAND_URL` - External randomness beacon URL.
- `CELLHAWK_ZMQ_URI` - ZMQ pub/sub URL (default: `tcp://127.0.0.1:5555`).

## 3. Log File Locations
- Core Logs: Stdout/Stderr (Managed by systemd or Docker).
- Panic Dumps: Tracing output gracefully traps panics to stderr.

## 4. Common Failure Modes
- **SDR Disconnected (`RtlSdrError`)**: The USB RTL-SDR dongle dropped. Resolution: Restart the pod to force a USB reset.
- **Redis Unreachable**: The swarm Danger Grid subscriber will back off. Resolution: Check the `redis-server` pod.

## 5. Required Linux Capabilities (Cgroups)
The agent requires `CAP_SYS_NICE` for real-time EKF scheduling and `CAP_IPC_LOCK` if mlock is enabled. Do not run as root.

## 6. Rollback Plan
To execute a rollback (Step 39):
1. `git revert HEAD --no-edit`
2. `cargo build --release`
3. `systemctl restart cellhawk-agent`
This takes < 5 minutes.
