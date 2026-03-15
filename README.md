# Multiplayer Bevy Template

> A production-ready multiplayer game template using Bevy + Lightyear

[![CI](https://github.com/BenDavies1218/Multiplayer-Template/workflows/CI/badge.svg)](https://github.com/BenDavies1218/Multiplayer-Template/actions)
[![Docker Server](https://github.com/BenDavies1218/Multiplayer-Template/workflows/Docker%20Server/badge.svg)](https://github.com/BenDavies1218/Multiplayer-Template/actions)
[![Docker Native](https://github.com/BenDavies1218/Multiplayer-Template/workflows/Docker%20Native/badge.svg)](https://github.com/BenDavies1218/Multiplayer-Template/actions)
[![Docker World Viewer](https://github.com/BenDavies1218/Multiplayer-Template/workflows/Docker%20World%20Viewer/badge.svg)](https://github.com/BenDavies1218/Multiplayer-Template/actions)
[![Deploy Web](https://github.com/BenDavies1218/Multiplayer-Template/workflows/Deploy%20Web/badge.svg)](https://github.com/BenDavies1218/Multiplayer-Template/actions)

An authoritative client-server multiplayer 3D game template built with [Bevy 0.18](https://bevyengine.org/), [Lightyear 0.26](https://github.com/cBournhonesque/lightyear) for networking/replication, and [Avian3d 0.5](https://github.com/Jondolf/avian) for physics. Clients use prediction and server reconciliation for responsive gameplay. Runs on native desktop, web (WASM), and Docker.

Use this template as a starting point for building multiplayer games with Bevy. It handles the hard parts — networked physics, client-side prediction, rollback, visual interpolation, and cross-platform transport — so you can focus on game logic.

## Quick Start

```bash
# Start the server
cargo dev-server

# In another terminal, start a client
cargo dev-native

# Or run the web client
cd apps/web && trunk serve

# Or use Docker
docker compose up -d  # Server on :5888, web client on :8080
```

### Controls

| Action             | Key          |
| ------------------ | ------------ |
| Move               | W/A/S/D      |
| Jump               | Space        |
| Sprint             | Left Shift   |
| Crouch             | Left Ctrl    |
| Prone              | C            |
| Fire               | Left Click   |
| Aim Down Sights    | Right Click  |
| Tactical Equipment | Middle Click |
| Reload             | R            |
| Melee              | V            |
| Interact           | E            |
| Lethal Equipment   | Q            |
| Primary Weapon     | 1            |
| Secondary Weapon   | 2            |
| Pause              | Escape       |

All bindings are configurable via `assets/config/game_client_config.json`. Gamepad is also supported.

## What's Included

- **Authoritative server** with client-side prediction and server reconciliation
- **Smooth interpolation** with configurable buffer for remote players
- **Rollback and correction** with visual error smoothing
- **Networked physics** via Avian3d with replication
- **Multiple transports** — UDP (native), WebSocket, and WebTransport
- **Cross-platform** — native desktop (Linux/macOS/Windows) and WASM web builds
- **JSON-driven configuration** — all game settings in `assets/config/`
- **Camera system** — first-person, third-person, and free-view modes
- **Dynamic objects** — data-driven interactables Trigger & Action architecture (doors, pickups, lights, animations).
- **World pipeline** — Blender-exported glTF/GLB with collision meshes, spawn points, zones, and triggers

## Documentation

Full documentation is available in the **[Multiplayer Bevy Template Book](book/src/SUMMARY.md)**.

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

MIT
