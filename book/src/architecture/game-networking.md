# game-networking

Shared networking protocol, replication, and movement logic used by both client and server.

## Sub-Plugin Architecture

`NetworkingPlugin` is a convenience wrapper that composes four independent sub-plugins. Apps that need fine-grained control can add sub-plugins individually instead.

| Plugin | Responsibility |
|--------|----------------|
| `ProtocolPlugin` | Lightyear component registration, prediction/rollback setup, input plugin |
| `RollbackPlugin` | Initializes speed-scaled rollback thresholds from `RollbackConfig` |
| `ProjectilePlugin` | `shoot_bullet` + `despawn_system` in `FixedUpdate` (chained) |
| `MovementPlugin` | Namespace for shared movement functions; systems are scheduled by consumers |

In addition to the sub-plugins, `NetworkingPlugin` registers `Transform`/`GlobalTransform` types and adds the Avian3d + Lightyear physics plugins with the correct disabled plugins.

## Modules

| Module | Purpose |
|--------|---------|
| `protocol` | Lightyear protocol registration, prediction and rollback setup. Shared type definitions (`CharacterAction`, `CameraOrientation`, `CrouchState`, marker components) are defined in `game-protocol` and re-exported here |
| `rollback` | Speed-scaled rollback thresholds and comparison functions registered with Lightyear |
| `rollback_plugin` | `RollbackPlugin` — initializes rollback config at startup |
| `config` | Environment variable loading (`Config`), shared networking settings (`SharedSettings`), helper functions |
| `movement` | `apply_character_movement()` — camera-relative deterministic movement shared by client and server |
| `movement_plugin` | `MovementPlugin` — namespace plugin for movement systems |
| `projectile_plugin` | `ProjectilePlugin` — projectile spawning and despawn lifecycle |
| `replication` | Modular replication systems organized by entity type |
| `replication::player` | `CharacterPhysicsBundle` — capsule collider + rigid body for player characters |
| `replication::projectile` | `shoot_bullet()` — projectile spawning with server replication and client pre-spawning |

## Key Types

- **`NetworkingPlugin`** — Convenience wrapper that adds all sub-plugins, physics, and type registration. Replaces the previous `SharedPlugin`.
- **`ProtocolPlugin`** — Registers all replicated components and the leafwing input plugin.
- **`RollbackPlugin`** — Stores rollback thresholds in a global static for comparison functions.
- **`ProjectilePlugin`** — Runs `shoot_bullet` and `despawn_system` in `FixedUpdate`.
- **`MovementPlugin`** — Empty plugin serving as a namespace and future extension point.
- **`CharacterAction`** — Input enum: `Move`, `Jump`, `Sprint`, `Crouch`, `Fire`, `Look`, plus combat, killstreak, and communication actions. Uses leafwing `InputManager`.
- **`CharacterPhysicsBundle`** — Capsule collider + dynamic rigid body with locked rotation axes.
- **`CameraOrientation`** — Yaw/pitch component (client authority, not predicted).
- **`CrouchState`** — Replicated crouch toggle with rollback support.
- **`Config`** — Environment variable configuration with `GameCoreConfig` fallback.
- **`SharedSettings`** — Protocol ID and private key for Netcode.io authentication.
- **`DespawnAfter`** — Timed entity despawn component for projectile lifetime.
- **Marker components** — `ProjectileMarker`, `FloorMarker`, `BlockMarker`, `ColorComponent`. (`CharacterMarker` is defined in `game-core` and re-exported here.)
- **`DynamicObject`** — Replicated marker with `object_type` and `object_id` for dynamic interactable objects. Registered in protocol for server->client replication.
- **`DynamicState`** — Replicated state of a dynamic object (current state string, togglable flag). Server-authoritative, registered in protocol for server->client replication.

## Extending Replication

The `replication/` module is organized by entity type for easy extension:

```
replication/
├── mod.rs           — Shared utilities (DespawnAfter, despawn_system)
├── player.rs        — Player character replication (CharacterPhysicsBundle)
└── projectile.rs    — Projectile replication (shoot_bullet)
```

To add new replicated entity types (e.g. health, inventory, vehicles):

1. Create a new file in `replication/` (e.g. `replication/health.rs`)
2. Define components and systems for that entity type
3. Add `pub mod health;` to `replication/mod.rs`
4. Register any new components in `protocol.rs` via `app.register_component::<T>()`
5. Add systems to `ProjectilePlugin` or create a new sub-plugin if they run on both client and server

## Dependencies

Depends on `game-core` for config types (`GameCoreConfig`, `MovementConfig`, `CharacterConfig`, etc.) and `CharacterMarker`. Depends on `game-protocol` for shared type definitions (`CharacterAction`, `CameraOrientation`, `CrouchState`, marker components), which are re-exported for backward compatibility. Uses Lightyear for networking, Avian3d for physics, and leafwing-input-manager for input.
