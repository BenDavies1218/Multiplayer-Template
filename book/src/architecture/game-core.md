# game-core

Shared game logic and configuration used by all crates. Does not contain networking protocol or movement logic (see `game-networking`).

## Modules

| Module | Purpose |
|--------|---------|
| `core_config` | Master `GameCoreConfig` resource containing all subsystem settings |
| `world` | Loads visual meshes from glTF/GLB, provides collision bundle and mesh helpers |
| `zones` | Loads zones + collision from a single GLB, spawn points, death/damage/trigger zones, collision detection |
| `character` | Character marker, hitbox loading from glTF/GLB, model identity, hitbox region components |
| `utils` | CLI argument parsing (`clap`), config file loading, log setup |

## Key Types

- **`GameCoreConfig`** — Top-level config loaded from `assets/config/game_core_config.json`. Contains nested configs: `NetworkingConfig`, `MovementConfig`, `CharacterConfig`, `WorldAssetsConfig`, `ZonesConfig`, `RollbackConfig`, `DebugColorsConfig`, `LoggingConfig`.
- **`CharacterMarker`** — Marker component for player character entities.
- **`CharacterModelId`** — Replicated string identifier for character visual model (e.g. `"default"`). Key into client's model catalog.
- **`HitboxRegion`** — Component on hitbox child entities. Contains region `name` and `base_damage` from glTF extras.
- **`CharacterHitboxData`** — Resource holding parsed hitbox regions from character hitbox GLB. Used to attach hitbox colliders on spawn.

> **Note:** The dynamic object system (`DynamicPlugin`, `DynamicObject`, `DynamicState`, etc.) has been extracted into the [`game-dynamic`](game-dynamic.md) crate.

## Plugins

- **`WorldPlugin`** — Loads world assets. Constructed with `WorldPluginConfig::server()`, `::client()`, or `::viewer()`.
- **`ZonePlugin`** — Processes zone meshes and runs collision detection. Server and viewer only.
- **`CharacterPlugin`** — Loads character hitbox GLBs from catalog, parses regions with damage attributes, provides `CharacterHitboxData` resource.

## Configuration

All config structs use `#[serde(default)]` so partial JSON files work. Loading is done via `load_config<T>(filename)` in `utils/config_loader.rs`, which falls back to `T::default()` if the file is missing.
