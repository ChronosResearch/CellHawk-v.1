# CELLHAWK Deployment Guide

This guide covers the deployment of the CELLHAWK triply-redundant navigation system in a production environment. 

## Prerequisites

- **Hardware**: Jetson Orin Nano (Edge Node), RTL-SDR v3 / HackRF, Intel Realsense D435.
- **OS**: Ubuntu 22.04 LTS (Host) or Alpine Linux (Container).
- **Network**: Redis Server (for Danger Grid), MAVLink connection to Flight Controller.
- **Docker**: Docker Engine 24.0+ and Docker Compose v2.

## Deployment Steps

### 1. Configure the Environment
Ensure your configuration is correctly set in `config.toml`. Never hardcode API keys or sensitive data.
Create a `.env` file based on `.env.example`:
```bash
cp .env.example .env
# Edit .env with your specific secrets
```

### 2. Build the Docker Image
Build the containerized edge node using the provided `Dockerfile`.
```bash
docker build -t cellhawk-agent:v1.0 .
```

### 3. Deploy via Docker Compose
Use the production docker-compose file which sets memory limits and drops privileges:
```bash
docker-compose -f docker-compose.prod.yml up -d
```

### 4. Verify System Health
Check the telemetry output and ensure all watchdogs are green.
```bash
curl http://localhost:8080/health
```

## Rollback Procedures

If an anomalous state is detected, the orchestrator's circuit breaker will open. 
To manually rollback to a previous version:
1. `docker-compose down`
2. Update `.env` to point to the previous stable tag (e.g. `cellhawk-agent:v0.9`).
3. `docker-compose -f docker-compose.prod.yml up -d`
