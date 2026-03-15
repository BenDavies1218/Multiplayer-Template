# game-protocol

Shared type definitions for the multiplayer protocol.

## Purpose

`game-protocol` is a leaf crate that contains the component, action, and marker type definitions shared across the networking stack. By isolating these types in their own crate, other crates (such as `game-core`) can depend on them without pulling in the full `game-networking` crate and its heavier transitive dependencies (Avian3d, full Lightyear client/server features, etc.).

## Key Types

| Type | Kind | Description |
|------|------|-------------|
| `CharacterAction` | Enum | Input action enum (movement, combat, communication, misc) with leafwing `Actionlike` impl |
| `CameraOrientation` | Component | Yaw/pitch for camera replication (client authority) |
| `CrouchState` | Component | Replicated crouch toggle |
| `ColorComponent` | Component | Serializable color wrapper |
| `FloorMarker` | Component | Marker for floor entities |
| `ProjectileMarker` | Component | Marker for projectile entities |
| `BlockMarker` | Component | Marker for block entities |

## Dependencies

Minimal dependency set — only what is needed for type definitions:

- `bevy` (component derives, `Reflect`)
- `lightyear` (replication derives)
- `leafwing-input-manager` (`Actionlike` trait)
- `serde` (serialization derives)

## Relationship to game-networking

`game-networking` depends on `game-protocol` and re-exports all its types via `pub use game_protocol::*`. This means existing code that imports from `game_networking::protocol` continues to work without changes. The Lightyear registration and plugin setup remain in `game-networking`.
