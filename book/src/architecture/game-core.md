# game-core

Shared game logic and configuration used by all crates. Does not contain networking protocol or movement logic (see `game-networking`).

## Modules

| Module | Purpose |
|--------|---------|
| `core_config` | Master `GameCoreConfig` resource containing all subsystem settings |
| `world` | Loads visual meshes from glTF/GLB, provides collision bundle and mesh helpers |
| `zones` | Loads zones + collision from a single GLB, spawn points, death/damage/trigger zones, collision detection |
| `dynamic` | Data-driven interactable objects (doors, pickups, lights, zones) with trigger/action system, entity type detection (mesh/light/empty/camera), procedural light effects (flicker/pulse/cycle/fixed), and mesh transform tweens (move/rotate/scale) |
| `character` | Character marker, hitbox loading from glTF/GLB, model identity, hitbox region components |
| `utils` | CLI argument parsing (`clap`), config file loading, log setup |

## Key Types

- **`GameCoreConfig`** — Top-level config loaded from `assets/config/game_core_config.json`. Contains nested configs: `NetworkingConfig`, `MovementConfig`, `CharacterConfig`, `WorldAssetsConfig`, `ZonesConfig`, `RollbackConfig`, `DebugColorsConfig`, `LoggingConfig`.
- **`CharacterMarker`** — Marker component for player character entities.
- **`CharacterModelId`** — Replicated string identifier for character visual model (e.g. `"default"`). Key into client's model catalog.
- **`HitboxRegion`** — Component on hitbox child entities. Contains region `name` and `base_damage` from glTF extras.
- **`CharacterHitboxData`** — Resource holding parsed hitbox regions from character hitbox GLB. Used to attach hitbox colliders on spawn.
- **`DynamicObject`** — Replicated marker component with `object_type`, `object_id`, and `entity_type: EntityType` (mesh/light/empty/camera) for interactable objects.
- **`DynamicState`** — Replicated state of a dynamic object (current state string, togglable flag).
- **`DynamicBehavior`** — Parsed trigger/action definitions from JSON config. Server + viewer only.
- **`ActiveLightEffects`** — Currently running procedural intensity and color effects on a light entity (flicker, pulse, cycle, fixed).
- **`DynamicTween`** — Active translation/rotation/scale tween on a dynamic object, with easing support.
- **`DynamicObjectRegistry`** — Resource mapping object IDs (Blender node names) to entities for cross-object targeting.

## Plugins

- **`WorldPlugin`** — Loads world assets. Constructed with `WorldPluginConfig::server()`, `::client()`, or `::viewer()`.
- **`ZonePlugin`** — Processes zone meshes and runs collision detection. Server and viewer only.
- **`CharacterPlugin`** — Loads character hitbox GLBs from catalog, parses regions with damage attributes, provides `CharacterHitboxData` resource.
- **`DynamicPlugin`** — Loads dynamic interactable objects from GLB, runs trigger detection and state actions. Constructed with `DynamicPluginConfig::server()`, `::client()`, or `::viewer()`.

## Configuration

All config structs use `#[serde(default)]` so partial JSON files work. Loading is done via `load_config<T>(filename)` in `utils/config_loader.rs`, which falls back to `T::default()` if the file is missing.
