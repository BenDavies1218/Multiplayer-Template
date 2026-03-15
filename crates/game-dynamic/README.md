# game-dynamic

Data-driven interactable object system for the multiplayer game.

## Overview

This crate provides the complete dynamic object pipeline: loading objects from Blender-exported GLB files, processing them into ECS entities, detecting triggers, executing actions, and running procedural effects.

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

## Plugin

**`DynamicPlugin`** -- Main plugin, constructed with `DynamicPluginConfig`:

- `DynamicPluginConfig::server()` -- Loads objects, runs trigger detection and state actions
- `DynamicPluginConfig::client()` -- Loads objects for visual rendering (light effects, mesh tweens)
- `DynamicPluginConfig::viewer()` -- Loads objects with debug visualization and local trigger/effect execution

## Dependencies

- `game-core` -- Config loading, world mesh helpers, character marker, debug config
- `game-protocol` -- Shared type definitions (future: `CharacterAction` for interact triggers)
