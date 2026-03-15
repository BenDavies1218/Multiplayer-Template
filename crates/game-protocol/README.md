# game-protocol

Shared type definitions for the multiplayer protocol.

This leaf crate contains component, action, and marker type definitions that are shared across the networking stack. By isolating these types in their own crate, other crates (such as `game-core`) can depend on them without pulling in the full `game-networking` crate and its heavier transitive dependencies.

## Key Types

- **`CharacterAction`** — Input action enum (movement, combat, communication, misc) with leafwing `Actionlike` impl.
- **`CameraOrientation`** — Yaw/pitch component for camera replication.
- **`CrouchState`** — Replicated crouch toggle.
- **`ColorComponent`** — Serializable color wrapper.
- **Marker components** — `FloorMarker`, `ProjectileMarker`, `BlockMarker`.

## Note

This crate only defines types. Lightyear registration, prediction setup, and plugin configuration remain in `game-networking`, which re-exports all types from this crate for backward compatibility.
