# Deployment Guide

> Production deployment guide for Multiplayer-Template

## Overview

The multiplayer template produces several deployable artifacts:

- **Server** runs in Docker or as a standalone binary, handling game state and replication
- **Native client** distributed as a binary download for desktop platforms
- **Web client** served as static files via nginx, S3+CloudFront, or any CDN
- **Assets** are separate from binaries and configurable via `asset_path` or the `ASSET_PATH` environment variable

## Server Deployment

### Docker Compose (Recommended)

The included `docker-compose.yml` starts the server and web client together:

```bash
docker compose up -d
```

This starts:

- Server on port 5888 (configurable via `SERVER_PORT`)
- Web client on port 8080

Assets are mounted read-only from the host `./assets` directory into `/data/assets` inside the container. The `ASSET_PATH` environment variable is set to `/data/assets` automatically.

Certificates for WebTransport are mounted from `./certificates`.

To customize, create a `.env` file alongside the compose file:

```env
SERVER_PORT=5888
RUST_LOG=info
```

### Standalone Binary

Build or download the server binary from the [Releases](https://github.com/BenDavies1218/Multiplayer-Template/releases) page, then run it directly:

```bash
# Set the asset path and start the server
ASSET_PATH=/opt/game/assets ./server server
```

Alternatively, configure `asset_path` in `game_core_config.json` instead of using the environment variable.

### Cloud Deployment

The server runs on any VPS or cloud instance with Docker support:

- Expose port **5888** for the game server (UDP/WebSocket/WebTransport)
- Optionally expose port **8080** for the web client
- Mount assets from persistent storage (e.g., an EBS volume or bind mount)
- For **WebTransport**: valid TLS certificates are required in production; self-signed certificates only work for local development

## Client Distribution

### Web Client

The web client is built with [Trunk](https://trunkrs.dev/) and outputs static files (HTML, JS, WASM):

```bash
cd apps/web
trunk build --release
```

The resulting `dist/` directory can be served from any static file host: nginx, S3+CloudFront, Vercel, Netlify, or similar. Assets are baked into the WASM build at compile time, so no separate asset directory is needed for the web client.

### Native Client

Native clients are available as pre-built binaries from the [Releases](https://github.com/BenDavies1218/Multiplayer-Template/releases) page, or can be built from source:

```bash
cargo build --release -p native
```

To run, point the client at an assets directory:

```bash
ASSET_PATH=./my-game-assets ./native-client -c 1
```

Or configure `asset_path` in `game_core_config.json`.

## Asset Management

### Directory Structure

```text
assets/
├── config/
│   ├── game_core_config.json
│   ├── game_client_config.json
│   ├── game_camera_config.json
│   └── game_server_config.json
├── models/
│   ├── example_world_visual.glb
│   ├── example_world_collision.glb
│   └── example_world_zones.glb
├── textures/
├── audio/
└── fonts/
```

### Configuring Asset Path

There are three ways to configure where assets are loaded from:

| Method | Example | Use Case |
|--------|---------|----------|
| Default | `../../assets` (relative) | Local development |
| Env var | `ASSET_PATH=/data/assets` | Docker containers |
| JSON config | `"asset_path": "/opt/game/assets"` | Production, per-platform |

Resolution order: `ASSET_PATH` env var > default (`../../assets`). After config loads, `asset_path` from `game_core_config.json` is used for the Bevy `AssetServer`.

### Swapping Game Content

To change worlds or models without rebuilding:

1. Replace files in the assets directory
2. Update paths in `game_core_config.json` if filenames changed
3. Restart the server and/or client

## Release Workflow

### Creating a Release

Push a version tag to trigger the GitHub Actions release workflow:

```bash
git tag v1.0.0
git push origin v1.0.0
```

This builds binaries for all supported platforms plus the WASM web client and creates a GitHub Release with all artifacts attached.

### Release Artifacts

| Artifact | Contents |
|----------|----------|
| `multiplayer-template-x86_64-unknown-linux-gnu.tar.gz` | server, native-client, world-viewer |
| `multiplayer-template-aarch64-unknown-linux-gnu.tar.gz` | server, native-client, world-viewer |
| `multiplayer-template-x86_64-apple-darwin.tar.gz` | server, native-client, world-viewer |
| `multiplayer-template-aarch64-apple-darwin.tar.gz` | server, native-client, world-viewer |
| `multiplayer-template-x86_64-pc-windows-msvc.zip` | server, native-client, world-viewer |
| `multiplayer-template-web.tar.gz` | WASM web client (static files) |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ASSET_PATH` | `../../assets` | Base path for configs and assets |
| `RUST_LOG` | `info` | Log level |
| `SERVER_HOST` | `127.0.0.1` | Server bind address |
| `SERVER_PORT` | `5888` | Server port |
| `DISPLAY` | - | X11 display (Linux GUI apps) |

## Ports

| Port | Service | Protocol |
|------|---------|----------|
| 5888 | Game server | UDP/WebSocket/WebTransport |
| 8080 | Web client (nginx) | HTTP |

## Security Notes

- WebTransport requires valid TLS certificates for production
- Self-signed certificates work for development only
- Certificates are stored in the `certificates/` directory, separate from assets
- Never commit private keys to version control
