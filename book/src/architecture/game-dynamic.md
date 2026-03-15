# game-dynamic

Data-driven interactable object system. Loads dynamic objects from Blender-exported GLB files and manages their lifecycle through triggers, actions, and procedural effects.

## Modules

| Module | Purpose |
|--------|---------|
| `types` | Core type definitions: `DynamicObject`, `DynamicState`, `DynamicBehavior`, `EntityType`, `TriggerType`, `ActionType`, config structs, light/tween effect types, `DynamicObjectRegistry` |
| `events` | `DynamicTriggerEvent` and `DynamicActionEvent` message types |
| `triggers` | Trigger detection systems: enter/exit collision, proximity interact, spawn, timer, delay, state change, target state change, action dispatch |
| `actions` | Server-side state mutation actions: toggle, set, enable, disable, collect, visibility |
| `light_effects` | Procedural light intensity and color effects: flicker, pulse, cycle, fixed |
| `mesh_effects` | Tweened transform animations: move, rotate, scale with easing |
| `loader` | Startup system to load dynamic GLB assets |
| `processor` | Processes loaded GLB nodes into ECS entities with components, colliders, lights, and debug meshes |
| `debug` | World viewer debug visualization overlay for dynamic objects |

## Key Types

- **`DynamicObject`** -- Replicated marker component with `object_type`, `object_id`, and `entity_type: EntityType` (mesh/light/empty/camera).
- **`DynamicState`** -- Replicated state of a dynamic object (current state string, togglable flag).
- **`DynamicBehavior`** -- Parsed trigger/action definitions from JSON config. Server + viewer only.
- **`ActiveLightEffects`** -- Currently running procedural intensity and color effects on a light entity (flicker, pulse, cycle, fixed).
- **`DynamicTween`** -- Active translation/rotation/scale tween on a dynamic object, with easing support.
- **`DynamicObjectRegistry`** -- Resource mapping object IDs (Blender node names) to entities for cross-object targeting.
- **`DynamicObjectsConfig`** -- Root config resource loaded from `dynamic_objects_config.json`.

## Plugin

**`DynamicPlugin`** -- Loads dynamic objects from GLB, runs trigger detection and state actions. Constructed with `DynamicPluginConfig`:

- `DynamicPluginConfig::server()` -- Triggers + state actions, no visuals
- `DynamicPluginConfig::client()` -- Visual effects only (light effects, mesh tweens)
- `DynamicPluginConfig::viewer()` -- Debug visualization + local trigger/effect execution

## Dependencies

- `game-core` -- Config loading utilities, world mesh helpers, character marker, debug color config
- `game-protocol` -- Shared type definitions

## Configuration

Dynamic objects are configured in `assets/config/dynamic_objects_config.json`. The config maps GLB node names to behavior definitions (triggers, actions, state, light info). See the [Dynamic Objects Guide](../guides/world-dynamic/dynamic-objects.md) for full documentation.
