# Dynamic Objects Guide

How to define interactive, data-driven game objects in Blender (doors, pickups, lights, switches) and export them for use in the game.

---

## How It Works

Dynamic objects are loaded from a GLB file (`world_dynamic.glb`) for meshes and positions. All behavior (triggers, actions, state) is defined in a separate JSON config file (`dynamic_objects_config.json`) — no Blender custom properties needed. The processor matches GLB node names to config entries and builds ECS entities accordingly. The server runs trigger detection and state mutations. Clients execute visual actions (animations, lights, text). State is server-authoritative and replicated via Lightyear.

**Key files:**

- `crates/game-core/src/dynamic/` — plugin, loader, processor, triggers, actions, types, debug
- `crates/game-client/src/dynamic_rendering.rs` — client-side visual action execution
- `tools/glb-parser/` — CLI tool to generate config templates from GLB files

**Asset paths:**
- `assets/models/world_dynamic.glb` (configured in `game_core_config.json`)
- `assets/config/dynamic_objects_config.json` (behavior config)

---

## Architecture

```text
world_dynamic.glb          dynamic_objects_config.json
  (meshes/positions)          (triggers/actions/state)
        │                              │
        └──────────┬───────────────────┘
                   ▼
          ┌─────────────────┐
          │   Processor      │  Matches node names → config
          │   (all modes)    │  DynamicObject, DynamicState, DynamicBehavior
          └────────┬────────┘
                   │
          ┌────────┴────────┐
          │                  │
          ▼                  ▼
   ┌──────────┐       ┌──────────────┐
   │  Server   │       │    Client     │
   │  Triggers │       │    Visuals    │
   │  Actions  │       │    (rendering)│
   └──────────┘       └──────────────┘
```

- **Server** — Detects triggers (collision, proximity, spawn), dispatches actions, mutates state
- **Client** — Receives replicated state, executes visual actions (lights, animations, text, sound)
- **Viewer** — Debug visualization of dynamic object colliders

---

## Trigger Types

| Type | Serde Name | Description |
|------|-----------|-------------|
| PlayerOnEnter | `playerOnEnter` | Player enters the object's sensor collider |
| PlayerOnExit | `playerOnExit` | Player exits the object's sensor collider |
| PlayerOnInteract | `playerOnInteract` | Player presses Interact within radius |
| PlayerOnShoot | `playerOnShoot` | A projectile hits the object |
| OnEntityHealth | `onEntityHealth` | Object's health drops below a threshold |
| OnEntitySpawn | `onEntitySpawn` | Fires once when the entity first spawns |

---

## Action Types

| Type | Serde Name | Side | Parameters |
|------|-----------|------|------------|
| PlayAnimation | `play_animation` | Client | `animation`, `speed` |
| StopAnimation | `stop_animation` | Client | — |
| SetLightIntensity | `set_light_intensity` | Client | `target`, `intensity`, `duration` |
| SetLightColor | `set_light_color` | Client | `target`, `color` (hex), `duration` |
| ToggleState | `toggle_state` | Server | — |
| Collect | `collect` | Server | — (despawns entity) |
| MoveTo | `move_to` | Server | `position`, `duration` |
| DisplayText | `display_text` | Client | `text` |
| HideText | `hide_text` | Client | — |
| PlaySound | `play_sound` | Client | `sound` |
| Enable | `enable` | Server | `target` (optional) |
| Disable | `disable` | Server | `target` (optional) |

---

## Config File Schema

All behavior is defined in `assets/config/dynamic_objects_config.json`. Each key in `nodes` matches a Blender node name from the GLB.

### Structure

```json
{
  "nodes": {
    "node_name": {
      "triggers": [...],
      "state": { ... }
    }
  }
}
```

### Trigger Definition

Each trigger owns an ordered list of actions that fire when the trigger activates:

```json
{
  "type": "playerOnEnter",
  "actions": [
    { "type": "display_text", "text": "Press E to open" }
  ]
}
```

Additional trigger-specific parameters can be added at the same level as `type` and `actions` — they are captured via `serde(flatten)`:

```json
{
  "type": "playerOnInteract",
  "radius": 2.0,
  "actions": [
    { "type": "play_animation", "animation": "door_open", "speed": 1.0 },
    { "type": "toggle_state" }
  ]
}
```

### Action Definition

Each action has a `type` and optional parameters captured as key-value pairs:

```json
{ "type": "set_light_intensity", "target": "room_light", "intensity": 8.0, "duration": 1.0 }
```

### State Configuration

```json
{
  "initial": "closed",
  "toggle": true
}
```

State pairs for `toggle_state`: `open`/`closed`, `on`/`off`, `active`/`idle`. Default initial state is `"idle"`.

---

## Full Example

A config file with a door, pickup, and light:

```json
{
  "nodes": {
    "door_01": {
      "triggers": [
        {
          "type": "playerOnEnter",
          "actions": [
            { "type": "display_text", "text": "Press E to open" }
          ]
        },
        {
          "type": "playerOnInteract",
          "radius": 2.0,
          "actions": [
            { "type": "play_animation", "animation": "door_open", "speed": 1.0 },
            { "type": "set_light_intensity", "target": "room_light", "intensity": 8.0, "duration": 1.0 },
            { "type": "toggle_state" }
          ]
        },
        {
          "type": "playerOnExit",
          "actions": [
            { "type": "hide_text" }
          ]
        }
      ],
      "state": { "initial": "closed", "toggle": true }
    },
    "pickup_health_01": {
      "triggers": [
        {
          "type": "playerOnEnter",
          "actions": [
            { "type": "play_sound", "sound": "pickup.ogg" },
            { "type": "collect" }
          ]
        }
      ]
    },
    "torch_01": {
      "triggers": [
        {
          "type": "onEntitySpawn",
          "actions": [
            { "type": "set_light_color", "target": "torch_light", "color": "#ff6600" },
            { "type": "set_light_intensity", "target": "torch_light", "intensity": 5.0 }
          ]
        }
      ]
    }
  }
}
```

---

## Blender Setup

### 1. Create a Dynamic Objects Collection

Create a dedicated collection in Blender (e.g., `World_dynamic`). This keeps dynamic objects separate from visual, collision, and zone meshes.

### 2. Add Dynamic Objects

For each dynamic object, add a mesh that defines its visual shape and collision area:

- **Doors/platforms**: Use the actual door mesh — the processor creates a sensor collider from it for enter/exit triggers
- **Pickups**: Small meshes (sphere, cube, custom model) at the pickup location
- **Lights**: An empty or small mesh at the light position — the `target` param references the light node by name

### 3. Name Your Objects

Node names become the `object_id` in the `DynamicObject` component and are used for cross-object `target` references and config file keys:

```text
door_01           → Config key "door_01"
pickup_health_01  → Config key "pickup_health_01"
switch_room_a     → Config key "switch_room_a"
room_light        → Referenced as target "room_light"
```

### 4. Generate Config Template

After exporting the GLB, run the parser to generate a config template:

```sh
cargo run -p glb-parser -- assets/models/world_dynamic.glb -o assets/config/dynamic_objects_config.json
```

This creates an entry for every node with empty triggers. Edit the config file to add behavior. Re-running the parser merges new nodes without overwriting existing config.

---

## Export Settings

```text
File → Export → glTF 2.0

Include:
  ✅ Selected Objects
  ❌ Custom Properties (behavior is in config file, not GLB)
  ❌ Cameras
  ❌ Punctual Lights (unless dynamic light sources are in this file)

Transform:
  ✅ +Y Up

Geometry:
  ✅ Apply Modifiers
  ✅ Normals
  ❌ Tangents
  ❌ Vertex Colors

Animation:
  ✅ Animations (if objects have baked animations)

Save as: world_dynamic.glb
```

---

## Server vs Client

| Environment | Loads Dynamic | Triggers | State Actions | Visual Actions | Debug |
|-------------|--------------|----------|---------------|----------------|-------|
| Server | Yes | Yes | Yes | No | No |
| Client | Yes | No | No | Yes | No |
| World Viewer | Yes | No | No | No | Yes |

The server is fully authoritative over dynamic object state. State changes are replicated to clients via Lightyear. Clients only execute visual effects.

---

## Debug Visualization

The world viewer includes debug visualization for dynamic objects.

**Toggle:** Press `D` to show/hide dynamic object debug meshes.

**Color:** Cyan (30% opacity) by default, configurable in `game_core_config.json` under `debug_colors.dynamic_object`.

Debug meshes are semi-transparent overlays rendered on top of dynamic object colliders. Only available in the world viewer (`DynamicPluginConfig::viewer()`).

**Key files:**

- `crates/game-core/src/dynamic/debug.rs` — `DynamicDebugSettings`, toggle and visibility systems
- `crates/game-core/src/dynamic/processor.rs` — spawns debug meshes alongside dynamic entities

---

## Cross-Object Targeting

Actions can reference other dynamic objects by name using the `target` parameter. The `DynamicObjectRegistry` maps node names to entities at load time.

Example: a switch that enables a door in another part of the map:

```json
{
  "nodes": {
    "switch_01": {
      "triggers": [
        {
          "type": "playerOnInteract",
          "radius": 1.5,
          "actions": [
            { "type": "enable", "target": "door_secret" },
            { "type": "set_light_intensity", "target": "door_secret_light", "intensity": 5.0 }
          ]
        }
      ]
    }
  }
}
```

---

## Components

| Component | Replicated | Description |
|-----------|-----------|-------------|
| `DynamicObject` | Yes | Marker with `object_type` and `object_id` |
| `DynamicState` | Yes | Current state string and togglable flag |
| `DynamicBehavior` | No | Parsed trigger/action definitions (server + viewer only) |
| `InteractionRadius` | No | Proximity radius for interact triggers |
| `DynamicEnabled` | No | Whether the object processes triggers |

---

## Best Practices

### Object Design

- Keep meshes simple — the processor creates sensor colliders from the mesh geometry
- Use meaningful node names for cross-object targeting
- One trigger can have multiple actions — compose effects instead of duplicating triggers
- Use `onEntitySpawn` for initialization (lights, starting animations)

### State Management

- Use `toggle_state` for binary states (open/closed, on/off)
- Set `"toggle": true` in state config for togglable objects
- State is replicated — clients react to changes via `sync_dynamic_state_visuals`

### Performance

- Sensor colliders are trimesh-based — keep geometry low-poly for trigger volumes
- Use `enable`/`disable` actions to deactivate objects that aren't needed
- Cross-object references resolve at load time via the registry

### Alignment

- Apply all transforms before export: select all → `Ctrl+A` → All Transforms
- Use the same origin point as your visual and collision meshes
- Dynamic object positions should match the visual world

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Dynamic objects not loading | Check `dynamic_path` in `game_core_config.json`, verify file exists at `assets/models/world_dynamic.glb` |
| Node not getting triggers | Verify the node name in the GLB matches the key in `dynamic_objects_config.json` exactly |
| Config not loading | Check `assets/config/dynamic_objects_config.json` exists and is valid JSON |
| Triggers not firing | Check server logs for "Created dynamic object" messages, verify object has `DynamicEnabled(true)` |
| Cross-object target not found | Verify the `target` name matches the exact Blender node name |
| State not toggling | Ensure `"toggle": true` in the state config and state is a recognized pair (open/closed, on/off, active/idle) |
| Debug meshes not showing | Press `D` to toggle, only works in world viewer |
| Light actions not working | Ensure the `target` name matches the light node name in `world_visual.glb` |
| Enter/exit not detecting | Verify the mesh has volume (not a single plane), check collider is created in logs |
| Parser not finding nodes | Ensure GLB is valid, run `cargo run -p glb-parser -- path/to/file.glb` to check output |
