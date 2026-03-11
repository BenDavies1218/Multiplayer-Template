# Multiplayer-Template

> A production-ready multiplayer game template using Bevy + Lightyear

[![CI](https://github.com/BenDavies1218/Multiplayer-Template/workflows/CI/badge.svg)](https://github.com/BenDavies1218/Multiplayer-Template/actions)
[![Docker Server](https://github.com/BenDavies1218/Multiplayer-Template/workflows/Docker%20Server/badge.svg)](https://github.com/BenDavies1218/Multiplayer-Template/actions)
[![Docker Native](https://github.com/BenDavies1218/Multiplayer-Template/workflows/Docker%20Native/badge.svg)](https://github.com/BenDavies1218/Multiplayer-Template/actions)
[![Docker World Viewer](https://github.com/BenDavies1218/Multiplayer-Template/workflows/Docker%20World%20Viewer/badge.svg)](https://github.com/BenDavies1218/Multiplayer-Template/actions)
[![Deploy Web](https://github.com/BenDavies1218/Multiplayer-Template/workflows/Deploy%20Web/badge.svg)](https://github.com/BenDavies1218/Multiplayer-Template/actions)

A multiplayer 3D character controller game template built with Bevy, featuring client-side prediction, server reconciliation, and smooth interpolation using the Lightyear networking library. Perfect for starting your multiplayer game project with a solid foundation.

## Features

- **Client-Side Prediction**: Immediate local player response with server reconciliation
- **Smooth Interpolation**: 100ms interpolation buffer for remote players
- **Rollback & Correction**: Visual error smoothing over multiple frames
- **Physics**: Avian3d physics engine with networked replication
- **Multiple Transports**: Support for UDP, WebSocket, and WebTransport
- **Cross-Platform**: Native desktop and WASM web builds
- **JSON Configuration**: All game settings driven by JSON config files in `assets/config/`
- **Camera Modes**: First-person, third-person, and free-view camera system

## Technology Stack

- **[Bevy 0.18](https://bevyengine.org/)**: Game engine
- **[Lightyear 0.26.4](https://github.com/cBournhonesque/lightyear)**: Networking and replication
- **[Avian3d 0.5](https://github.com/Jondolf/avian)**: Physics engine
- **[Leafwing Input Manager 0.20](https://github.com/leafwing-studios/leafwing-input-manager)**: Input handling
- **[Trunk](https://trunkrs.dev/)**: WASM build tool

## Project Structure

```text
multiplayer-bevy/
├── Cargo.toml              # Workspace root
├── README.md               # This file
├── .cargo/config.toml      # Cargo aliases (dev-native, dev-server, etc.)
│
├── crates/                 # Library crates
│   ├── game-core/          # Shared protocol, game logic, and config [→ README](crates/game-core/README.md)
│   ├── game-client/        # Client-specific code (prediction, rendering, input) [→ README](crates/game-client/README.md)
│   ├── game-server/        # Server-specific code (authority, spawning) [→ README](crates/game-server/README.md)
│   └── game-camera/        # Camera system (first-person, third-person, free-view) [→ README](crates/game-camera/README.md)
│
├── apps/                   # Binary applications
│   ├── server/             # Dedicated server [→ README](apps/server/README.md)
│   ├── native/             # Native desktop client [→ README](apps/native/README.md)
│   ├── world-viewer/       # Standalone world viewer [→ README](apps/world-viewer/README.md)
│   └── web/                # WASM web client [→ README](apps/web/README.md)
│
├── assets/                 # Game assets
│   ├── models/             # World meshes (visual, collision, zones)
│   ├── textures/
│   ├── audio/
│   ├── fonts/
│   └── config/             # JSON configuration files
│       ├── game_core_config.json
│       ├── game_client_config.json
│       ├── game_camera_config.json
│       └── game_server_config.json
│
└── certificates/           # WebTransport TLS certificates
```

## Quick Start

### Prerequisites

- Rust 1.93+ (edition 2024)
- For web builds: [Trunk](https://trunkrs.dev/) (`cargo install trunk`)

### Build Everything

```bash
# Build all workspace crates and binaries
cargo build --all

# Build in release mode (Production Optimised)
cargo build --all --release
```

### Run the Server

```bash
cargo run -p server -- server

# Or with dynamic linking for faster dev builds
cargo dev-server
```

The server listens on port **5888** with UDP by default. Transport and port are configured in `assets/config/game_server_config.json` and `assets/config/game_core_config.json`.

### Run a Native Client

```bash
cargo run -p native -- client -c 1

# Or with dynamic linking for faster dev builds
cargo dev-native
```

### Run a Web Client

```bash
cd apps/web
trunk serve
```

Open <http://localhost:8080> in your browser.

### Controls

- **W/A/S/D**: Move
- **Space**: Jump
- **Left Shift**: Sprint
- **C**: Crouch
- **Q**: Shoot
- **Left Click**: Grab cursor
- **Escape**: Release cursor

## Configuration

All game settings are driven by JSON config files in `assets/config/`:

- **`game_core_config.json`** — Asset path, networking (host, port, tick rate), movement, physics, world assets, zones
- **`game_client_config.json`** — Window settings, input bindings, rendering, transport
- **`game_camera_config.json`** — Camera modes (first-person, third-person, free-view), sensitivity
- **`game_server_config.json`** — Projectile settings, spawning, transport type, certificate SANs

## Docker Deployment

### Using Docker Compose (Recommended)

```bash
docker compose up -d
```

This starts:

- Server on port 5888
- Web client on port 8080

Assets are mounted from `./assets` into containers via volume mounts. To use custom assets, place them in the `assets/` directory or set `ASSET_PATH` in the compose environment.

### Building Individual Images

```bash
# Build server
docker build -f Dockerfile.server -t multiplayer-server .

# Build web client
docker build -f Dockerfile.web -t multiplayer-web .

# Build native client
docker build -f Dockerfile.native -t multiplayer-native .

# Build world viewer
docker build -f Dockerfile.world-viewer -t multiplayer-world-viewer .
```

### Using Pre-built Images

```bash
# Pull and run server
docker pull ghcr.io/BenDavies1218/multiplayer-template-server:latest
docker run -p 5888:5888 \
  -v ./certificates:/certificates \
  -v ./assets:/data/assets \
  -e ASSET_PATH=/data/assets \
  ghcr.io/BenDavies1218/multiplayer-template-server:latest

# Pull and run web client
docker pull ghcr.io/BenDavies1218/multiplayer-template-web:latest
docker run -p 8080:80 ghcr.io/BenDavies1218/multiplayer-template-web:latest

# Pull and run native client (requires X11 forwarding for GUI)
docker pull ghcr.io/BenDavies1218/multiplayer-template-native:latest
docker run -e DISPLAY=$DISPLAY -v /tmp/.X11-unix:/tmp/.X11-unix \
  -v ./assets:/data/assets -e ASSET_PATH=/data/assets \
  ghcr.io/BenDavies1218/multiplayer-template-native:latest -c 1

# Pull and run world viewer (requires X11 forwarding for GUI)
docker pull ghcr.io/BenDavies1218/multiplayer-template-world-viewer:latest
docker run -e DISPLAY=$DISPLAY -v /tmp/.X11-unix:/tmp/.X11-unix \
  -v ./assets:/data/assets -e ASSET_PATH=/data/assets \
  ghcr.io/BenDavies1218/multiplayer-template-world-viewer:latest
```

See [DEPLOYMENT.md](DEPLOYMENT.md) for production deployment guide.

## Release Binaries

Pre-built binaries are available on the [Releases](../../releases) page, triggered by version tags (`v*`).

### Platforms

| Platform            | Target                         |
| ------------------- | ------------------------------ |
| Linux x86_64        | `x86_64-unknown-linux-gnu`     |
| Linux ARM64         | `aarch64-unknown-linux-gnu`    |
| macOS Intel         | `x86_64-apple-darwin`          |
| macOS Apple Silicon | `aarch64-apple-darwin`         |
| Windows             | `x86_64-pc-windows-msvc`       |
| Web (WASM)          | Separate `web.tar.gz` artifact |

### Each archive contains

- `server` — Dedicated game server
- `native-client` — Native desktop client
- `world-viewer` — Standalone world viewer tool

### Usage

Download the archive for your platform, extract, and run with your assets:

```bash
# Set the asset path and run
ASSET_PATH=./my-game-assets ./server server
ASSET_PATH=./my-game-assets ./native-client -c 1
```

Or configure `asset_path` in your `game_core_config.json`.

## Development Workflow

### Cargo Aliases

Fast development builds with dynamic linking are available via `.cargo/config.toml`:

```bash
cargo dev-native   # Run native client with dynamic linking (client -c 1)
cargo dev-server   # Run server with dynamic linking
cargo dev-viewer   # Run world viewer with dynamic linking
```

### Building Specific Targets

```bash
# Build library crates
cargo build -p game-core
cargo build -p game-client
cargo build -p game-server
cargo build -p game-camera

# Build binaries
cargo build -p server
cargo build -p native
cargo build -p web
```

### Running Tests

```bash
cargo test --all
```

### Adding Dependencies

Add dependencies to the workspace root `Cargo.toml`:

```toml
[workspace.dependencies]
your-crate = "1.0"
```

Then reference in individual crates:

```toml
[dependencies]
your-crate.workspace = true
```

## Architecture

### Crate Separation

- **[game-core](crates/game-core/README.md)**: Contains the protocol definition, shared game logic, physics bundles, world/zone loading, configuration types, and common utilities. Used by both client and server.
- **[game-client](crates/game-client/README.md)**: Client-specific code including input handling, prediction, rendering, and transport setup.
- **[game-server](crates/game-server/README.md)**: Server authority logic, player spawning, projectile management, and server transport setup.
- **[game-camera](crates/game-camera/README.md)**: Flexible camera system supporting first-person, third-person, and free-view modes with configurable sensitivity and smoothing.

### Networking Features

- **Protocol**: Component-based replication with Lightyear
- **Prediction**: Full client-side prediction for local player
- **Interpolation**: 100ms buffer for smooth remote player movement
- **Rollback**: Physics state rollback and correction for mispredictions
- **Frame Interpolation**: Visual smoothing between fixed update ticks

## App-Specific Documentation

- [Server Documentation](apps/server/README.md)
- [Native Client Documentation](apps/native/README.md)
- [World Viewer Documentation](apps/world-viewer/README.md)
- [Web Client Documentation](apps/web/README.md)

## Performance Tips

- Use release builds for better performance: `cargo build --release`
- The first build will be slow; subsequent builds are much faster
- Use `cargo dev-*` aliases for faster incremental dev builds with dynamic linking
- Web builds benefit significantly from `trunk serve --release`

## Troubleshooting

### Certificate Errors

If you see WebTransport certificate errors:

- Ensure `certificates/` directory exists at the repository root
- Generate certificates if missing (see server README)

### WASM Build Errors

If `trunk build` fails:

- Ensure you're in the `apps/web` directory
- Check that Trunk.toml has correct configuration
- Verify getrandom backend is set for WASM

## License

MIT OR Apache-2.0
