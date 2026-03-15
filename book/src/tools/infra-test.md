# infra-test

Infrastructure load testing tool that spins up a game server and N headless clients using Docker containers pulled from GHCR. Includes a Prometheus + Grafana + cAdvisor monitoring stack for measuring server resource usage under load.

## Purpose

Determine how much CPU/memory a server needs to support N concurrent players. Run increasing numbers of headless clients and observe when the server's tick rate degrades or resources saturate.

## Prerequisites

- Docker and Docker Compose installed
- Images published to GHCR (happens automatically on push to `main`):
  - `ghcr.io/bendavies1218/multiplayer-template-server:latest`
  - `ghcr.io/bendavies1218/multiplayer-template-native:latest`

## Usage

```sh
cd tools/infra-test

# Start 1 server + 10 headless clients + monitoring stack
./run.sh --clients 10

# Scale to 20 clients
./run.sh --clients 20

# View container logs
./run.sh --logs

# Tear everything down
./run.sh --down
```

## Monitoring

Once running, the monitoring stack is available at:

| Service | URL | Description |
|---------|-----|-------------|
| Grafana | [localhost:3000](http://localhost:3000) | Pre-built load test dashboard |
| Prometheus | [localhost:9090](http://localhost:9090) | Metrics query interface |
| cAdvisor | [localhost:8080](http://localhost:8080) | Per-container resource stats |

Grafana is pre-configured with anonymous admin access — no login required.

### Load Test Dashboard

The pre-built Grafana dashboard (`http://localhost:3000/d/load-test/load-test-dashboard`) includes:

- **Server CPU Usage (%)** — how much CPU the server container consumes
- **Server Memory Usage (MB)** — server memory footprint
- **Network RX/TX bytes/sec** — network traffic per container
- **All Container CPU comparison** — side-by-side CPU across server and all clients
- **Running client count** — number of active client containers

## Headless Client Mode

The native client supports a `--headless` flag that runs without a window or GPU:

```sh
# Run locally (no Docker)
cargo run -p native -- client --headless

# With a specific client ID
cargo run -p native -- client --headless -c 42
```

When `--headless` is used without `-c`, a random client ID is generated automatically. This is how the Docker containers run — each replica gets a unique ID.

### What headless mode does

- Uses `MinimalPlugins` instead of `DefaultPlugins` (no window, no rendering pipeline)
- Loads world assets for collision/zone detection
- Runs the full networking stack (connection, prediction, input, lifecycle)
- Skips all rendering: camera, skybox, character models, visual interpolation, cursor, animation

### What headless mode does NOT do

- No GPU required — runs on headless servers
- No player input simulation (yet) — clients connect and idle
- No visual output — monitor via server diagnostics logs

## Server Diagnostics

The server diagnostics log (every 10 seconds when `enable_diagnostics: true`) includes metrics useful for load testing:

```
[DIAGNOSTICS] Clients: 10 | 🟢 FPS: 64.0 | 🟢 Frame: 15.62ms | Entities: 42 | CPU: 8.3% | RAM: 124MB | 🟢 Rollbacks: 0.2/s | 🟢 Depth: 1.5 | 🟢 Tick: 64.0/64.0 Hz (100%)
```

Key metrics for load testing:

| Metric | What to watch |
|--------|--------------|
| **Clients** | Connected client count |
| **CPU** | Server process CPU usage |
| **Tick** | Actual vs target tick rate — degradation here means the server can't keep up |
| **FPS** | Should stay at or near the target tick rate |
| **RAM** | Memory growth as clients connect |

### Tick Rate Health Indicators

| Status | Condition |
|--------|-----------|
| 🟢 Good | >= 95% of target |
| 🟠 Warning | 80-95% of target |
| 🔴 Critical | < 80% of target |

## Architecture

```
tools/infra-test/
├── docker-compose.yml          # GHCR image definitions
├── prometheus.yml               # Prometheus scrape config
├── run.sh                       # CLI entrypoint
└── grafana/
    ├── dashboards/
    │   └── load-test.json       # Pre-built dashboard
    └── provisioning/
        ├── dashboards/
        │   └── dashboard.yml    # Dashboard auto-provisioning
        └── datasources/
            └── prometheus.yml   # Prometheus datasource
```

## Typical Workflow

1. Push code to `main` — CI builds and publishes Docker images to GHCR
2. Run `./run.sh --clients 5` to establish a baseline
3. Incrementally increase: `./run.sh --clients 10`, `--clients 20`, `--clients 50`
4. Watch the Grafana dashboard for:
   - Server CPU approaching 100%
   - Tick rate dropping below 95%
   - Memory growth patterns
5. Use the results to size your production server
