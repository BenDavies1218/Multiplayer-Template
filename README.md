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
- **Dynamic Objects**: Data-driven interactable objects (doors, pickups, lights) defined via Blender custom properties

## Technology Stack

- **[Bevy 0.18](https://bevyengine.org/)**: Game engine
- **[Lightyear 0.26.4](https://github.com/cBournhonesque/lightyear)**: Networking and replication
- **[Avian3d 0.5](https://github.com/Jondolf/avian)**: Physics engine
- **[Leafwing Input Manager 0.20](https://github.com/leafwing-studios/leafwing-input-manager)**: Input handling
- **[Trunk](https://trunkrs.dev/)**: WASM build tool

## Quick Start

```bash
# Start the server
cargo dev-server

# In another terminal, start a client
cargo dev-native

# Or run the web client
cd apps/web && trunk serve
```

### Controls

- **W/A/S/D**: Move
- **Space**: Jump
- **Left Shift**: Sprint
- **C**: Crouch
- **Q**: Shoot
- **Left Click**: Grab cursor
- **Escape**: Release cursor

## Documentation

Full documentation is available in the **[Multiplayer Bevy Template Book](book/src/SUMMARY.md)**.

The book covers:

- **[Getting Started](book/src/getting-started/installation.md)** — Installation, quick start, configuration
- **[Architecture](book/src/architecture/overview.md)** — Crate structure, plugin pattern, networking
- **[Applications](book/src/apps/server.md)** — Server, native client, web client, world viewer
- **[Guides](book/src/guides/deployment.md)** — Deployment, meshes, lighting, zones, dynamic objects
- **[Contributing](book/src/contributing.md)** — Development setup, workflow, code style

To build and serve the book locally:

```bash
cargo install mdbook
cd book
mdbook serve --open
```

## Docker Quick Start

```bash
docker compose up -d
```

This starts the server on port 5888 and the web client on port 8080.

## Release Binaries

Pre-built binaries are available on the [Releases](../../releases) page, triggered by version tags (`v*`).

| Platform            | Target                         |
| ------------------- | ------------------------------ |
| Linux x86_64        | `x86_64-unknown-linux-gnu`     |
| Linux ARM64         | `aarch64-unknown-linux-gnu`    |
| macOS Intel         | `x86_64-apple-darwin`          |
| macOS Apple Silicon | `aarch64-apple-darwin`         |
| Windows             | `x86_64-pc-windows-msvc`       |
| Web (WASM)          | Separate `web.tar.gz` artifact |

## License

MIT OR Apache-2.0
