# CELLHAWK Production Runbook

## Overview
This runbook provides operational procedures for monitoring and maintaining the CELLHAWK fleet.

## Monitoring

### Prometheus Metrics
The edge node exposes a Prometheus metrics endpoint at `:9090/metrics`.
**Key Metrics to Monitor**:
- `ekf_innovation_error_meters`: Must stay below 5.0m in Tier 1, 42.0m in Tier 2.
- `handover_latency_ms`: Alert if this exceeds 250ms.
- `swarm_bandwidth_bps`: Must remain under 4000 (4 kbps).
- `watchdog_resets_total`: Any value > 0 warrants investigation.

### Logging
Logs are structured JSON output to stdout (captured by Docker). Use `jq` to query logs:
```bash
docker logs cellhawk-agent | jq 'select(.level == "ERROR")'
```

## Alerts & Responses

### 1. Alert: `CircuitBreakerOpen`
**Trigger**: Component (e.g. Vision SLAM) crashed repeatedly within a 1-minute window.
**Response**: 
- The agent will fallback to the remaining active sensors.
- If safe, command `RTL` (Return to Launch).
- Retrieve core dumps from the persistent volume for debugging.

### 2. Alert: `JnrThresholdExceeded`
**Trigger**: Jamming-to-Noise Ratio spikes above 19 dB.
**Response**:
- Expected behavior in a contested environment. Ensure the transition to Tier 2/3 was successful by checking `handover_latency_ms`.

### 3. Alert: `SwarmCoordinationLoss`
**Trigger**: Redis heartbeat from publisher lost for > 5 seconds.
**Response**:
- Drones will assume standalone operation.
- Danger Grid is stale. Proceed with extreme caution and consider increasing flight altitude.
