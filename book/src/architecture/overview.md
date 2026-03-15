# Architecture Overview

Authoritative client-server multiplayer 3D game built with Bevy 0.18, Lightyear 0.26 (networking/replication), and Avian3d 0.5 (physics). Clients use prediction + server reconciliation.

## Crate Separation

```text
┌────────────────┐
│  game-protocol │  (shared type definitions:
│                │   CharacterAction, markers,
│                │   CameraOrientation, etc.)
└───────┬────────┘
        │
        ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────────┐
│  game-core   │◄─│ game-dynamic │◄─│  game-networking  │
│  (config,    │  │ (triggers,   │  │  (protocol reg.,  │
│   world,     │  │  actions,    │  │   movement,       │
│   zones)     │  │  effects)    │  │   replication)    │
└──────┬───────┘  └──────────────┘  └────────┬─────────┘
       │         ┌────────────┼────────────┐
       │         │            │            │
       ▼         ▼            ▼            ▼
  ┌──────────┐ ┌──────────┐ ┌──────────┐
  │game-camera│ │game-client│ │game-server│
  └──────────┘ └──────────┘ └──────────┘
```

- **game-protocol** — Shared type definitions (actions, markers, components) used across the networking stack. Leaf crate with minimal dependencies.
- **game-core** — Shared config, world/zone loading, character definitions. Used by all apps.
- **game-dynamic** — Data-driven interactable object system (triggers, actions, light effects, mesh tweens, debug). Depends on `game-core` and `game-protocol`.
- **game-networking** — Shared networking protocol registration, replication, movement logic. Depends on `game-core`, `game-protocol`, and `game-dynamic`.
- **game-client** — Client systems: prediction, rendering, input, transport setup.
- **game-server** — Server systems: authoritative simulation, player management, spawning.
- **game-camera** — Camera modes: first-person, third-person, free-view.

## Key Architectural Rules

- Server is authoritative for game state. Clients predict locally and reconcile.
- Movement logic (`apply_character_movement()`) lives in `game-networking` and is shared by both client and server.
- All config is JSON-driven from `assets/config/`. Config structs use `#[serde(default)]` for partial files.
- World assets are Blender-exported glTF/GLB: visual mesh, zones+collision mesh (using name prefixes: `collision_`, `spawn_`, `deathzone_`, `damage_`, `trigger_`).
- Transport is runtime-configurable: UDP (native), WebSocket, WebTransport.

## Plugin Pattern

Each crate exports plugins via `lib.rs`. Apps compose them in order:

1. `NetworkingPlugin` from `game-networking` (protocol, physics, replication)
2. `WorldPlugin` (visual asset loading — has `server()`, `client()`, `viewer()` modes)
3. `ZonePlugin` (collision + zones — has `server()`, `client()`, `viewer()` modes)
4. Domain plugin (`ClientPlugin` or `ServerPlugin`)

## Component Conventions

- Marker components: `CharacterMarker`, `ProjectileMarker`, `FloorMarker`, `BlockMarker`
- Replicated components derive: `Serialize, Deserialize, Clone, Debug, PartialEq`
- Register with `app.register_component::<T>(ChannelDirection::ServerToClient)`

## System Scheduling

- `FixedUpdate`: Input handling, movement, physics (both client and server)
- `FixedPostUpdate`: Zone detection (server)
- `Update`: Visual updates, camera (client)

## Networking (Lightyear)

- Protocol defined in `crates/game-networking/src/protocol.rs`
- Input actions via `CharacterAction` enum with leafwing `InputManager`
- Rollback thresholds configured in `GameCoreConfig`
- Connection spawned via `spawn_client_connection_from_config()` / `spawn_server_connection_from_config()`
