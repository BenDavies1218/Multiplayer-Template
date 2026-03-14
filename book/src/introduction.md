# Multiplayer Bevy Template

> A production-ready multiplayer game template using Bevy + Lightyear

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

## Project Structure

```text
multiplayer-bevy/
├── Cargo.toml              # Workspace root
├── .cargo/config.toml      # Cargo aliases (dev-native, dev-server, etc.)
│
├── crates/                 # Library crates
│   ├── game-core/          # Shared protocol, game logic, and config
│   ├── game-client/        # Client-specific code (prediction, rendering, input)
│   ├── game-server/        # Server-specific code (authority, spawning)
│   └── game-camera/        # Camera system (first-person, third-person, free-view)
│
├── apps/                   # Binary applications
│   ├── server/             # Dedicated server
│   ├── native/             # Native desktop client
│   ├── world-viewer/       # Standalone world viewer
│   └── web/                # WASM web client
│
├── assets/                 # Game assets
│   ├── models/             # World meshes (visual, collision, zones, dynamic)
│   ├── textures/
│   ├── audio/
│   ├── fonts/
│   └── config/             # JSON configuration files
│
└── certificates/           # WebTransport TLS certificates
```
