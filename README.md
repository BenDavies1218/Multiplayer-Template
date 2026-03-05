# Multiplayer-Template

> A production-ready multiplayer game template using Bevy + Lightyear

[![CI](https://github.com/BenDavies1218/Multiplayer-Template/workflows/CI/badge.svg)](https://github.com/BenDavies1218/Multiplayer-Template/actions)
[![Docker Server](https://github.com/BenDavies1218/Multiplayer-Template/workflows/Docker%20Server/badge.svg)](https://github.com/BenDavies1218/Multiplayer-Template/actions)
[![Deploy Web](https://github.com/BenDavies1218/Multiplayer-Template/workflows/Deploy%20Web/badge.svg)](https://github.com/BenDavies1218/Multiplayer-Template/actions)

A multiplayer 3D character controller game template built with Bevy, featuring client-side prediction, server reconciliation, and smooth interpolation using the Lightyear networking library. Perfect for starting your multiplayer game project with a solid foundation.

## Features

- **Client-Side Prediction**: Immediate local player response with server reconciliation
- **Smooth Interpolation**: 100ms interpolation buffer for remote players (industry standard)
- **Rollback & Correction**: Visual error smoothing over multiple frames
- **Physics**: Avian3d physics engine with networked replication
- **Multiple Transports**: Support for UDP, WebSocket, and WebTransport
- **Cross-Platform**: Native desktop and WASM web builds

## Technology Stack

- **[Bevy 0.18](https://bevyengine.org/)**: Game engine
- **[Lightyear 0.26.4](https://github.com/cBournhonesque/lightyear)**: Networking and replication
- **[Avian3d 0.5](https://github.com/Jondolf/avian)**: Physics engine
- **[Leafwing Input Manager 0.20](https://github.com/leafwing-studios/leafwing-input-manager)**: Input handling
- **[Trunk](https://trunkrs.dev/)**: WASM build tool

## Project Structure

```
multiplayer-bevy/
├── Cargo.toml              # Workspace root
├── README.md               # This file
├── .gitignore
│
├── crates/                 # Library crates
│   ├── game-core/          # Shared protocol and game logic
│   ├── game-client/        # Client-specific code
│   └── game-server/        # Server-specific code
│
├── apps/                   # Binary applications
│   ├── server/             # Dedicated server [→ README](apps/server/README.md)
│   ├── native/             # Native desktop client [→ README](apps/native/README.md)
│   └── web/                # WASM web client [→ README](apps/web/README.md)
│
├── assets/                 # Game assets
│   ├── models/
│   ├── textures/
│   ├── audio/
│   ├── fonts/
│   └── config/
│
└── certificates/           # WebTransport certificates
```

## Quick Start

### Prerequisites

- Rust 1.88+ (edition 2024)
- For web builds: [Trunk](https://trunkrs.dev/) (`cargo install trunk`)

### Build Everything

```bash
# Build all workspace crates and binaries
cargo build --all

# Build in release mode (much faster)
cargo build --all --release
```

### Run the Server

```bash
cargo run -p server -- server
```

The server listens on port 5000 with WebTransport by default.

### Run a Native Client

```bash
cargo run -p native -- client --client-id 1
```

**Controls:**
- WASD: Move
- Space: Jump
- Left Click: Shoot

### Run a Web Client

```bash
cd apps/web
trunk serve
```

Open http://localhost:8080 in your browser.

## Docker Deployment

### Using Docker Compose (Recommended)

```bash
docker-compose up -d
```

This starts:
- Server on port 5888
- Web client on port 8080

### Using Pre-built Images

```bash
# Pull and run server
docker pull ghcr.io/BenDavies1218/multiplayer-template-server:latest
docker run -p 5888:5888 -v ./certificates:/certificates ghcr.io/BenDavies1218/multiplayer-template-server:latest

# Pull and run web client
docker pull ghcr.io/BenDavies1218/multiplayer-template-web:latest
docker run -p 8080:80 ghcr.io/BenDavies1218/multiplayer-template-web:latest
```

See [DEPLOYMENT.md](DEPLOYMENT.md) for production deployment guide.

## Development Workflow

### Building Specific Targets

```bash
# Build library crates
cargo build -p game-core
cargo build -p game-client
cargo build -p game-server

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

- **game-core**: Contains the protocol definition, shared game logic, physics bundles, and common utilities. Used by both client and server.
- **game-client**: Client-specific code including input handling, prediction, and rendering.
- **game-server**: Server authority logic, player spawning, and game state management.

### Networking Features

- **Protocol**: Component-based replication with Lightyear
- **Prediction**: Full client-side prediction for local player
- **Interpolation**: 100ms buffer for smooth remote player movement
- **Rollback**: Physics state rollback and correction for mispredictions
- **Frame Interpolation**: Visual smoothing between fixed update ticks

## App-Specific Documentation

- [Server Documentation](apps/server/README.md)
- [Native Client Documentation](apps/native/README.md)
- [Web Client Documentation](apps/web/README.md)

## Performance Tips

- Use release builds for better performance: `cargo build --release`
- The first build will be slow; subsequent builds are much faster
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

### Import Errors

If you see "use of undeclared crate" errors:
- Verify the crate is in the workspace dependencies
- Check import paths use correct crate names (`game_core::`, not `crate::`)

## License

MIT OR Apache-2.0

## Credits

Based on the [Lightyear examples](https://github.com/cBournhonesque/lightyear) by Charles Bournhonesque.
