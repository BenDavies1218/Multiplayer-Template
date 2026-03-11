# game-core

Shared game logic and protocol definitions used by both client and server.

## Modules

| Module | Purpose |
|--------|---------|
| `core_config` | Master `GameCoreConfig` resource containing all subsystem settings |
| `networking` | Lightyear protocol registration, shared components, replication setup |
| `movement` | `apply_character_movement()` — camera-relative movement shared by client and server |
| `world` | Loads visual/collision meshes from glTF/GLB, converts to Avian3d colliders |
| `zones` | Spawn points, death zones, damage zones, trigger zones with collision detection |
| `character` | Character hitbox loading from glTF/GLB, model identity, hitbox region components |
| `utils` | CLI argument parsing (`clap`), config file loading, log setup |

## Key Types

- **`GameCoreConfig`** — Top-level config loaded from `assets/config/game_core_config.json`. Contains nested configs: `NetworkingConfig`, `MovementConfig`, `CharacterConfig`, `WorldAssetsConfig`, `ZonesConfig`, `RollbackConfig`, `DebugColorsConfig`, `LoggingConfig`.
- **`CharacterAction`** — Input enum: `Move`, `Jump`, `Sprint`, `Crouch`, `Shoot`, `Look`. Uses leafwing `InputManager`.
- **`CharacterPhysicsBundle`** — Capsule collider + dynamic rigid body with locked rotation axes.
- **Marker components** — `CharacterMarker`, `ProjectileMarker`, `FloorMarker`, `BlockMarker`.
- **`CameraOrientation`** — Yaw/pitch component (client authority, not predicted).
- **`CrouchState`** — Replicated crouch toggle with rollback support.
- **`CharacterModelId`** — Replicated string identifier for character visual model (e.g. `"default"`). Key into client's model catalog.
- **`HitboxRegion`** — Component on hitbox child entities. Contains region `name` and `base_damage` from glTF extras.
- **`CharacterHitboxData`** — Resource holding parsed hitbox regions from character hitbox GLB. Used to attach hitbox colliders on spawn.

## Plugins

- **`SharedPlugin`** — Registers protocol components, adds physics (Avian3d), shared movement systems.
- **`WorldPlugin`** — Loads world assets. Constructed with `WorldPluginConfig::server()`, `::client()`, or `::viewer()`.
- **`ZonePlugin`** — Processes zone meshes and runs collision detection. Server and viewer only.
- **`CharacterPlugin`** — Loads character hitbox GLBs from catalog, parses regions with damage attributes, provides `CharacterHitboxData` resource.

## Configuration

All config structs use `#[serde(default)]` so partial JSON files work. Loading is done via `load_config<T>(filename)` in `utils/config_loader.rs`, which falls back to `T::default()` if the file is missing.
