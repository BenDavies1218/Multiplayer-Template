# Dynamic Objects Guide

How to define interactive, data-driven game objects in Blender (doors, pickups, lights, switches, zones) and export them for use in the game.

---

## How It Works

Dynamic objects are loaded from a GLB file (`world_dynamic.glb`) containing meshes, lights, empties, and cameras. All behavior (triggers, actions, state) is defined in a separate JSON config file (`dynamic_objects_config.json`). The glb-parser tool detects each node's **entity type** (mesh, light, empty, camera) from the GLB structure and generates a config template with type metadata. The processor matches GLB node names to config entries, spawns ECS entities with the correct components (including Bevy light components for light-type nodes), and wires up triggers and procedural effects. The server runs trigger detection and state mutations. Clients execute visual actions (light effects, mesh tweens, animations, text, sound). State is server-authoritative and replicated via Lightyear.

**Key files:**

- `crates/game-core/src/dynamic/` — plugin, loader, processor, triggers, actions, types, light_effects, mesh_effects, debug
- `crates/game-client/src/dynamic_rendering.rs` — client-side visual action execution
- `tools/glb-parser/` — CLI tool to generate type-aware config templates from GLB files

**Asset paths:**
- `assets/models/world_dynamic.glb` (configured in `game_core_config.json`)
- `assets/config/dynamic_objects_config.json` (behavior config)

---

## Architecture

```text
world_dynamic.glb              dynamic_objects_config.json
  (meshes/lights/empties)        (triggers/actions/state)
        |                                  |
        +---> glb-parser (detects types) --+
        |                                  |
        +----------------+-----------------+
                         v
                +-----------------+
                |   Processor      |  Matches node names -> config
                |   (all modes)    |  Detects entity type (mesh/light/empty/camera)
                |                  |  Spawns light components, ActiveLightEffects
                +---------+-------+
                          |
                +---------+----------+
                |                    |
                v                    v
         +----------+         +--------------+
         |  Server   |         |    Client     |
         |  Triggers |         |  Light effects|
         |  Actions  |         |  Mesh tweens  |
         |  State    |         |  Animations   |
         +----------+         +--------------+
```

- **Server** -- Detects triggers (collision, proximity, spawn, timer, delay, state change), dispatches actions, mutates state
- **Client** -- Receives replicated state, executes visual actions (light effects, mesh tweens, animations, text, sound)
- **Viewer** -- Debug visualization of dynamic object colliders, runs light effects and mesh tweens

---

## Entity Types

The processor detects the entity type of each node from the GLB structure. This determines what components are spawned and what actions are available.

| Type | Detected When | Use For |
|------|--------------|---------|
| `mesh` | Node has a mesh attached | Doors, platforms, pickups, visual objects |
| `light` | Node has `KHR_lights_punctual` extension | Torches, lamps, alarm lights, disco lights |
| `empty` | Node has no mesh, light, or camera | Trigger zones, spawn markers, invisible switches |
| `camera` | Node has a camera attached | Cutscene cameras (future) |

### How detection works

The glb-parser reads the GLB binary, extracts the JSON chunk, and checks each node:
1. If the node has a `KHR_lights_punctual` extension reference, it is a **light** and the tool extracts `light_info` (type, color, intensity) from the extension data.
2. If the node has a `mesh` index, it is a **mesh**.
3. If the node has a `camera` index, it is a **camera**.
4. Otherwise, it is an **empty**.

### glb-parser output format

The parser generates config entries with `type` and optional `light_info`:

```json
{
  "nodes": {
    "torch_light_01": {
      "type": "light",
      "light_info": {
        "light_type": "point",
        "color": [1.0, 0.8, 0.3],
        "intensity": 800.0
      },
      "triggers": []
    },
    "door_01": {
      "type": "mesh",
      "triggers": []
    },
    "zone_entrance": {
      "type": "empty",
      "triggers": []
    }
  }
}
```

Light-type entities get Bevy light components (`PointLight`, `SpotLight`, or `DirectionalLight`) spawned automatically from `light_info`, plus an `ActiveLightEffects` component for the procedural light effects system.

---

## Trigger Reference

Triggers define when actions fire. Each trigger has a `type` and an `actions` array. Some triggers accept additional parameters.

### Player Triggers

| JSON Name | Description | Parameters |
|-----------|-------------|------------|
| `playerOnEnter` | Player enters the object's sensor collider | -- |
| `playerOnExit` | Player exits the object's sensor collider | -- |
| `playerOnInteract` | Player presses Interact within radius | `radius` (float) |
| `playerOnShoot` | A projectile hits the object | -- |

**playerOnEnter** -- Fires when a player character enters the sensor collider created from the node's mesh geometry. Requires a mesh on the node.

```json
{
  "type": "playerOnEnter",
  "actions": [
    { "type": "display_text", "text": "Press E to open" }
  ]
}
```

**playerOnExit** -- Fires when a player character exits the sensor collider.

```json
{
  "type": "playerOnExit",
  "actions": [
    { "type": "hide_text" }
  ]
}
```

**playerOnInteract** -- Fires when a player presses the Interact key within the specified radius.

```json
{
  "type": "playerOnInteract",
  "radius": 2.0,
  "actions": [
    { "type": "toggle_state" }
  ]
}
```

**playerOnShoot** -- Fires when a projectile collides with the object.

```json
{
  "type": "playerOnShoot",
  "actions": [
    { "type": "set_state", "value": "broken" }
  ]
}
```

### Entity Triggers

| JSON Name | Description | Parameters |
|-----------|-------------|------------|
| `onEntitySpawn` | Fires once when the entity first spawns | -- |
| `onEntityHealth` | Object's health drops below a threshold | `threshold` (float) |

**onEntitySpawn** -- Fires once immediately after the entity is created. Use for initialization (starting light effects, initial animations).

```json
{
  "type": "onEntitySpawn",
  "actions": [
    { "type": "start_light_effect", "effect": "flicker", "min": 400.0, "max": 800.0, "speed": 3.0 }
  ]
}
```

**onEntityHealth** -- Fires when the entity's health drops below a threshold.

```json
{
  "type": "onEntityHealth",
  "threshold": 50.0,
  "actions": [
    { "type": "start_light_effect", "effect": "pulse", "min": 0.0, "max": 1000.0, "speed": 5.0 }
  ]
}
```

### State Triggers

| JSON Name | Description | Parameters |
|-----------|-------------|------------|
| `onStateChange` | Fires when the entity's own state transitions | `to` (string, optional) |
| `onTargetStateChange` | Fires when another entity's state changes | `target` (string), `to` (string, optional) |

**onStateChange** -- Fires whenever this entity's `DynamicState` changes. Optionally filter with the `to` parameter to only fire when transitioning to a specific state.

```json
{
  "type": "onStateChange",
  "to": "open",
  "actions": [
    { "type": "move_to", "offset": [0.0, 3.0, 0.0], "duration": 1.5, "easing": "ease_in_out" }
  ]
}
```

**onTargetStateChange** -- Fires when another entity (identified by `target`) changes state. Useful for chaining: a switch toggles its own state, which triggers actions on a door.

```json
{
  "type": "onTargetStateChange",
  "target": "switch_01",
  "to": "on",
  "actions": [
    { "type": "start_light_effect", "effect": "fixed", "intensity": 800.0 }
  ]
}
```

### Time Triggers

| JSON Name | Description | Parameters |
|-----------|-------------|------------|
| `onTimer` | Fires on a repeating interval | `interval` (float, seconds), `repeat` (bool, default true) |
| `onDelay` | Fires once after a delay from spawn | `delay` (float, seconds) |

**onTimer** -- Fires repeatedly at a fixed interval. The `DynamicTimer` component is attached at spawn.

```json
{
  "type": "onTimer",
  "interval": 5.0,
  "actions": [
    { "type": "toggle_state" }
  ]
}
```

**onDelay** -- Fires once after a delay from spawn, then the `DynamicDelay` component is removed.

```json
{
  "type": "onDelay",
  "delay": 3.0,
  "actions": [
    { "type": "start_light_effect", "effect": "pulse", "min": 200.0, "max": 600.0, "speed": 2.0 }
  ]
}
```

---

## Action Reference

Actions are the effects that fire when a trigger activates. Each action has a `type` and optional parameters.

### Universal Actions

These work on any entity type.

| JSON Name | Side | Parameters | Description |
|-----------|------|------------|-------------|
| `toggle_state` | Server | -- | Toggle between state pairs (open/closed, on/off, active/idle) |
| `set_state` | Server | `value` | Set state to an arbitrary string |
| `enable` | Server | `target` (optional) | Enable trigger processing on this or target entity |
| `disable` | Server | `target` (optional) | Disable trigger processing on this or target entity |
| `collect` | Server | -- | Despawn the entity (for pickups) |
| `display_text` | Client | `text` | Show text overlay to the triggering player |
| `hide_text` | Client | -- | Remove the text overlay |
| `play_sound` | Client | `sound` | Play an audio clip |
| `set_visibility` | Server | `visible` (bool) | Show or hide the entity |
| `start_light_effect` | Client | `effect`, `channel`, ... | Start a procedural light effect (see Light Effects API) |
| `stop_light_effect` | Client | `channel` (optional) | Stop a running light effect (see Light Effects API) |

**toggle_state** -- Toggles between recognized state pairs. Requires `"toggle": true` in state config.

```json
{ "type": "toggle_state" }
```

**set_state** -- Sets the state to any arbitrary value. Does not require `"toggle": true`.

```json
{ "type": "set_state", "value": "broken" }
```

**enable / disable** -- Enable or disable trigger processing. Without `target`, acts on self. With `target`, acts on the named entity.

```json
{ "type": "enable", "target": "door_secret" }
```

```json
{ "type": "disable" }
```

**collect** -- Despawns the entity. Use for pickups.

```json
{ "type": "collect" }
```

**display_text / hide_text** -- Show or hide a text overlay for the triggering player.

```json
{ "type": "display_text", "text": "Press E to open" }
```

```json
{ "type": "hide_text" }
```

**play_sound** -- Play an audio clip by filename.

```json
{ "type": "play_sound", "sound": "pickup.ogg" }
```

**set_visibility** -- Show or hide the entity's visual representation.

```json
{ "type": "set_visibility", "visible": false }
```

### Light Effect Actions

See the [Light Effects API](#light-effects-api) section for full documentation.

| JSON Name | Side | Description |
|-----------|------|-------------|
| `start_light_effect` | Client | Start a procedural light effect |
| `stop_light_effect` | Client | Stop a running light effect |

```json
{ "type": "start_light_effect", "effect": "flicker", "min": 400.0, "max": 800.0, "speed": 3.0 }
```

```json
{ "type": "stop_light_effect", "channel": "intensity" }
```

### Mesh Transform Actions

See the [Mesh Transform API](#mesh-transform-api) section for full documentation.

| JSON Name | Side | Parameters | Description |
|-----------|------|------------|-------------|
| `move_to` | Client | `offset`, `duration`, `easing` | Tween translation to a position |
| `rotate_to` | Client | `rotation`, `duration`, `easing` | Tween rotation to euler angles (degrees) |
| `scale_to` | Client | `scale`, `duration`, `easing` | Tween scale to a target |
| `set_material_color` | Client | `color` | Set the material base color |

```json
{ "type": "move_to", "offset": [0.0, 3.0, 0.0], "duration": 1.5, "easing": "ease_in_out" }
```

```json
{ "type": "rotate_to", "rotation": [0.0, 90.0, 0.0], "duration": 1.0, "easing": "ease_out" }
```

```json
{ "type": "scale_to", "scale": [2.0, 2.0, 2.0], "duration": 0.5 }
```

```json
{ "type": "set_material_color", "color": "#ff0000" }
```

### Legacy Actions

Kept for backward compatibility. Prefer `start_light_effect` / `stop_light_effect` for new configs.

| JSON Name | Side | Parameters | Description |
|-----------|------|------------|-------------|
| `play_animation` | Client | `animation`, `speed` | Play a named animation clip |
| `stop_animation` | Client | -- | Stop the current animation |
| `set_light_intensity` | Client | `target`, `intensity`, `duration` | Set a light's intensity by name |
| `set_light_color` | Client | `target`, `color` (hex) | Set a light's color by name |

```json
{ "type": "play_animation", "animation": "door_open", "speed": 1.0 }
```

```json
{ "type": "stop_animation" }
```

```json
{ "type": "set_light_intensity", "target": "room_light", "intensity": 800.0 }
```

```json
{ "type": "set_light_color", "target": "room_light", "color": "#ff6600" }
```

### Camera Actions (Future)

| JSON Name | Side | Parameters | Description |
|-----------|------|------------|-------------|
| `activate_camera` | Client | -- | Switch to this entity's camera |
| `deactivate_camera` | Client | -- | Return to the player camera |

```json
{ "type": "activate_camera" }
```

```json
{ "type": "deactivate_camera" }
```

---

## Light Effects API

The light effects system provides procedural, time-based effects for light-type entities. Effects are applied via the `start_light_effect` and `stop_light_effect` actions and are ticked every frame by the `tick_light_effects` system.

Light effects operate on two independent **channels**:
- **intensity** -- Controls the light's brightness (intensity for point/spot, illuminance for directional)
- **color** -- Controls the light's color

You can run one effect on each channel simultaneously (e.g., flicker intensity + fixed color).

### Effect Types

#### flicker

Random, organic intensity variation using multi-frequency noise. Good for torches, candles, broken lights.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `effect` | string | -- | Must be `"flicker"` |
| `channel` | string | `"intensity"` | `"intensity"` or `"color"` |
| `min` | float | 0.0 | Minimum intensity value |
| `max` | float | 1.0 | Maximum intensity value |
| `speed` | float | 1.0 | How fast the flicker animates |

**Intensity flicker (torch):**

```json
{
  "type": "start_light_effect",
  "effect": "flicker",
  "min": 400.0,
  "max": 800.0,
  "speed": 3.0
}
```

**Color flicker (electrical fault):**

```json
{
  "type": "start_light_effect",
  "effect": "flicker",
  "channel": "color",
  "min_color": [0.5, 0.5, 1.0],
  "max_color": [1.0, 1.0, 1.0],
  "speed": 8.0
}
```

#### pulse

Smooth sinusoidal oscillation. Good for alarms, magical effects, breathing lights.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `effect` | string | -- | Must be `"pulse"` |
| `channel` | string | `"intensity"` | `"intensity"` or `"color"` |
| `min` | float | 0.0 | Minimum intensity value |
| `max` | float | 1.0 | Maximum intensity value |
| `speed` | float | 1.0 | Oscillation speed (cycles per ~6.28 seconds at speed 1.0) |

**Intensity pulse (alarm):**

```json
{
  "type": "start_light_effect",
  "effect": "pulse",
  "min": 0.0,
  "max": 1000.0,
  "speed": 4.0
}
```

**Color pulse (magical glow):**

```json
{
  "type": "start_light_effect",
  "effect": "pulse",
  "channel": "color",
  "min_color": [0.0, 0.0, 0.5],
  "max_color": [0.0, 0.5, 1.0],
  "speed": 1.5
}
```

#### cycle

Smoothly interpolates through an ordered list of colors. Good for disco lights, status indicators, mood lighting.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `effect` | string | -- | Must be `"cycle"` |
| `colors` | array of [r,g,b] | -- | List of colors to cycle through (linear RGB, 0.0-1.0) |
| `speed` | float | 1.0 | How fast to cycle (colors per second) |

The cycle always targets the **color** channel (the `channel` parameter is ignored).

```json
{
  "type": "start_light_effect",
  "effect": "cycle",
  "colors": [
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0]
  ],
  "speed": 0.5
}
```

#### fixed

Sets a constant intensity and/or color. Good for initialization, on/off states, combining with other effects.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `effect` | string | -- | Must be `"fixed"` (or omit `effect` entirely) |
| `intensity` | float | -- | Fixed intensity value (sets intensity channel) |
| `color` | [r,g,b] | -- | Fixed color (sets color channel) |

You can set one or both channels in a single fixed action.

**Set intensity only:**

```json
{
  "type": "start_light_effect",
  "effect": "fixed",
  "intensity": 500.0
}
```

**Set color only:**

```json
{
  "type": "start_light_effect",
  "effect": "fixed",
  "color": [1.0, 0.4, 0.0]
}
```

**Set both:**

```json
{
  "type": "start_light_effect",
  "effect": "fixed",
  "intensity": 800.0,
  "color": [1.0, 0.8, 0.3]
}
```

### Combining Effects

You can run one intensity effect and one color effect simultaneously. Each `start_light_effect` call only replaces the channel it targets.

**Flickering torch with warm color:**

```json
[
  { "type": "start_light_effect", "effect": "flicker", "min": 400.0, "max": 800.0, "speed": 3.0 },
  { "type": "start_light_effect", "effect": "fixed", "color": [1.0, 0.6, 0.2] }
]
```

The first action sets the intensity channel to flicker. The second sets the color channel to a fixed warm orange. Both run simultaneously.

**Pulsing alarm with red color cycling:**

```json
[
  { "type": "start_light_effect", "effect": "pulse", "min": 100.0, "max": 1000.0, "speed": 4.0 },
  { "type": "start_light_effect", "effect": "cycle", "colors": [[1.0, 0.0, 0.0], [0.5, 0.0, 0.0]], "speed": 2.0 }
]
```

### Stopping Effects

Use `stop_light_effect` to clear running effects. You can target a specific channel or clear both.

**Stop intensity only:**

```json
{ "type": "stop_light_effect", "channel": "intensity" }
```

**Stop color only:**

```json
{ "type": "stop_light_effect", "channel": "color" }
```

**Stop all effects (both channels):**

```json
{ "type": "stop_light_effect" }
```

---

## Mesh Transform API

The mesh transform system provides tweened (animated) changes to an entity's position, rotation, and scale. Tweens are driven by the `DynamicTween` component and ticked every frame by the `tick_mesh_tweens` system.

### Actions

#### move_to

Tweens the entity's translation from its current position to a target position.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `offset` | [x, y, z] | -- | Target world position |
| `duration` | float | 1.0 | Duration in seconds |
| `easing` | string | `"linear"` | Easing function |

```json
{ "type": "move_to", "offset": [0.0, 3.0, 0.0], "duration": 1.5, "easing": "ease_in_out" }
```

#### rotate_to

Tweens the entity's rotation from its current orientation to target euler angles (in degrees). Uses quaternion slerp internally for smooth interpolation.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `rotation` | [x, y, z] | -- | Target euler angles in degrees (XYZ order) |
| `duration` | float | 1.0 | Duration in seconds |
| `easing` | string | `"linear"` | Easing function |

```json
{ "type": "rotate_to", "rotation": [0.0, 90.0, 0.0], "duration": 1.0, "easing": "ease_out" }
```

#### scale_to

Tweens the entity's scale from its current scale to a target scale.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `scale` | [x, y, z] | -- | Target scale |
| `duration` | float | 1.0 | Duration in seconds |
| `easing` | string | `"linear"` | Easing function |

```json
{ "type": "scale_to", "scale": [2.0, 2.0, 2.0], "duration": 0.5, "easing": "ease_in" }
```

### Easing Functions

| Value | Description |
|-------|-------------|
| `linear` | Constant speed (default) |
| `ease_in` | Starts slow, accelerates (quadratic) |
| `ease_out` | Starts fast, decelerates (quadratic) |
| `ease_in_out` | Slow start and end, fast middle (quadratic) |

### Door Example with State

A door that slides up when opened and back down when closed, driven by state changes:

```json
{
  "door_01": {
    "type": "mesh",
    "triggers": [
      {
        "type": "playerOnInteract",
        "radius": 2.0,
        "actions": [
          { "type": "toggle_state" }
        ]
      },
      {
        "type": "onStateChange",
        "to": "open",
        "actions": [
          { "type": "move_to", "offset": [0.0, 3.0, 0.0], "duration": 1.5, "easing": "ease_in_out" }
        ]
      },
      {
        "type": "onStateChange",
        "to": "closed",
        "actions": [
          { "type": "move_to", "offset": [0.0, 0.0, 0.0], "duration": 1.5, "easing": "ease_in_out" }
        ]
      }
    ],
    "state": { "initial": "closed", "toggle": true }
  }
}
```

The player interacts to toggle state. The `onStateChange` triggers react to the new state and start the appropriate tween.

---

## State System

Dynamic objects can have state that is replicated from server to clients. State is a simple string value.

### Configuration

```json
{
  "state": {
    "initial": "closed",
    "toggle": true
  }
}
```

- `initial` -- Starting state string. Default is `"idle"`.
- `toggle` -- Whether `toggle_state` works on this object.

### State Pairs for toggle_state

| State A | State B |
|---------|---------|
| `open` | `closed` |
| `on` | `off` |
| `active` | `idle` |

Any other state string toggles to `"active"`.

### set_state

Set state to any arbitrary value (not limited to pairs):

```json
{ "type": "set_state", "value": "broken" }
```

### Chaining with onStateChange

The `onStateChange` trigger fires whenever state changes, making it the primary way to drive visual effects from state:

```json
{
  "triggers": [
    {
      "type": "playerOnInteract",
      "radius": 1.5,
      "actions": [
        { "type": "toggle_state" }
      ]
    },
    {
      "type": "onStateChange",
      "to": "on",
      "actions": [
        { "type": "start_light_effect", "effect": "fixed", "intensity": 800.0 }
      ]
    },
    {
      "type": "onStateChange",
      "to": "off",
      "actions": [
        { "type": "stop_light_effect" }
      ]
    }
  ],
  "state": { "initial": "off", "toggle": true }
}
```

### Cross-Object State Chaining with onTargetStateChange

A light that reacts when a remote switch is toggled:

```json
{
  "alarm_light": {
    "type": "light",
    "light_info": { "light_type": "point", "color": [1.0, 0.0, 0.0], "intensity": 0.0 },
    "triggers": [
      {
        "type": "onTargetStateChange",
        "target": "alarm_switch",
        "to": "on",
        "actions": [
          { "type": "start_light_effect", "effect": "pulse", "min": 0.0, "max": 1000.0, "speed": 4.0 },
          { "type": "start_light_effect", "effect": "fixed", "color": [1.0, 0.0, 0.0] }
        ]
      },
      {
        "type": "onTargetStateChange",
        "target": "alarm_switch",
        "to": "off",
        "actions": [
          { "type": "stop_light_effect" }
        ]
      }
    ]
  }
}
```

---

## Config Schema

Full example showing all entity types and features:

```json
{
  "nodes": {
    "torch_01": {
      "type": "light",
      "light_info": {
        "light_type": "point",
        "color": [1.0, 0.8, 0.3],
        "intensity": 600.0
      },
      "triggers": [
        {
          "type": "onEntitySpawn",
          "actions": [
            { "type": "start_light_effect", "effect": "flicker", "min": 400.0, "max": 800.0, "speed": 3.0 },
            { "type": "start_light_effect", "effect": "fixed", "color": [1.0, 0.6, 0.2] }
          ]
        }
      ]
    },
    "door_01": {
      "type": "mesh",
      "triggers": [
        {
          "type": "playerOnEnter",
          "actions": [
            { "type": "display_text", "text": "Press E to open" }
          ]
        },
        {
          "type": "playerOnExit",
          "actions": [
            { "type": "hide_text" }
          ]
        },
        {
          "type": "playerOnInteract",
          "radius": 2.0,
          "actions": [
            { "type": "toggle_state" }
          ]
        },
        {
          "type": "onStateChange",
          "to": "open",
          "actions": [
            { "type": "move_to", "offset": [0.0, 3.0, 0.0], "duration": 1.5, "easing": "ease_in_out" },
            { "type": "play_sound", "sound": "door_open.ogg" }
          ]
        },
        {
          "type": "onStateChange",
          "to": "closed",
          "actions": [
            { "type": "move_to", "offset": [0.0, 0.0, 0.0], "duration": 1.5, "easing": "ease_in_out" },
            { "type": "play_sound", "sound": "door_close.ogg" }
          ]
        }
      ],
      "state": { "initial": "closed", "toggle": true }
    },
    "pickup_health_01": {
      "type": "mesh",
      "triggers": [
        {
          "type": "onEntitySpawn",
          "actions": [
            { "type": "start_light_effect", "effect": "pulse", "min": 0.5, "max": 1.5, "speed": 2.0 }
          ]
        },
        {
          "type": "playerOnEnter",
          "actions": [
            { "type": "play_sound", "sound": "pickup.ogg" },
            { "type": "collect" }
          ]
        }
      ]
    },
    "alarm_light_01": {
      "type": "light",
      "light_info": {
        "light_type": "point",
        "color": [1.0, 0.0, 0.0],
        "intensity": 0.0
      },
      "triggers": [
        {
          "type": "onTargetStateChange",
          "target": "alarm_switch",
          "to": "on",
          "actions": [
            { "type": "start_light_effect", "effect": "pulse", "min": 0.0, "max": 1000.0, "speed": 4.0 },
            { "type": "start_light_effect", "effect": "fixed", "color": [1.0, 0.0, 0.0] }
          ]
        },
        {
          "type": "onTargetStateChange",
          "target": "alarm_switch",
          "to": "off",
          "actions": [
            { "type": "stop_light_effect" }
          ]
        }
      ]
    },
    "zone_entrance": {
      "type": "empty",
      "triggers": [
        {
          "type": "playerOnEnter",
          "actions": [
            { "type": "display_text", "text": "Entering the Dungeon" }
          ]
        },
        {
          "type": "playerOnExit",
          "actions": [
            { "type": "hide_text" }
          ]
        }
      ]
    },
    "disco_light_01": {
      "type": "light",
      "light_info": {
        "light_type": "spot",
        "color": [1.0, 1.0, 1.0],
        "intensity": 500.0
      },
      "triggers": [
        {
          "type": "onEntitySpawn",
          "actions": [
            {
              "type": "start_light_effect",
              "effect": "cycle",
              "colors": [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 1.0, 0.0],
                [1.0, 0.0, 1.0]
              ],
              "speed": 0.8
            },
            {
              "type": "start_light_effect",
              "effect": "pulse",
              "min": 200.0,
              "max": 800.0,
              "speed": 2.0
            }
          ]
        }
      ]
    },
    "alarm_switch": {
      "type": "mesh",
      "triggers": [
        {
          "type": "playerOnInteract",
          "radius": 1.5,
          "actions": [
            { "type": "toggle_state" }
          ]
        }
      ],
      "state": { "initial": "off", "toggle": true }
    }
  }
}
```

---

## glb-parser

The CLI tool reads a GLB file, detects each node's entity type, extracts light metadata, and generates a `dynamic_objects_config.json` template.

### Usage

```sh
# Generate to stdout
cargo run -p glb-parser -- assets/models/world_dynamic.glb

# Generate to file (creates new or merges with existing)
cargo run -p glb-parser -- assets/models/world_dynamic.glb -o assets/config/dynamic_objects_config.json
```

### Merge mode

When the output file already exists, the parser runs in merge mode:
- Existing nodes keep their triggers, state, and config
- New nodes from the GLB are added with empty triggers
- Nodes in the config but missing from the GLB get a warning
- Existing nodes typed as `"empty"` are updated if the GLB now has better type info (e.g., a light was added)

### Type detection output

The parser prints a summary to stderr:

```
GLB nodes: 8, new: 3, updated: 1, total config nodes: 10
  empty: 1
  light: 4
  mesh: 3
```

---

## Blender Setup

### 1. Create a Dynamic Objects Collection

Create a dedicated collection in Blender (e.g., `World_dynamic`). This keeps dynamic objects separate from visual, collision, and zone meshes.

### 2. Add Dynamic Objects

For each dynamic object, add the appropriate Blender object type:

- **Doors/platforms** -- Use the actual door mesh. The processor creates a sensor collider from it for enter/exit triggers.
- **Pickups** -- Small meshes (sphere, cube, custom model) at the pickup location.
- **Lights** -- Add a Blender light (Point, Spot, or Sun/Area) at the desired position. The exporter writes `KHR_lights_punctual` data that the parser reads.
- **Zones/triggers** -- Use an Empty at the trigger location. The empty becomes an entity you can attach enter/exit triggers to (requires a mesh for the collider, or use interact radius).
- **Cameras** -- Add a Blender camera for future cutscene support.

### 3. Name Your Objects

Node names become the `object_id` in the `DynamicObject` component and are used for cross-object `target` references and config file keys:

```text
door_01           -> Config key "door_01"
torch_light_01    -> Config key "torch_light_01" (type: light)
pickup_health_01  -> Config key "pickup_health_01"
zone_entrance     -> Config key "zone_entrance" (type: empty)
alarm_switch      -> Config key "alarm_switch"
alarm_light_01    -> Referenced via "target": "alarm_switch"
```

### 4. Generate Config Template

After exporting the GLB, run the parser to generate a type-aware config template:

```sh
cargo run -p glb-parser -- assets/models/world_dynamic.glb -o assets/config/dynamic_objects_config.json
```

This creates an entry for every node with the detected type, light_info (for lights), and empty triggers. Edit the config file to add behavior. Re-running the parser merges new nodes without overwriting existing config.

---

## Export Settings

```text
File -> Export -> glTF 2.0

Include:
  Selected Objects
  Custom Properties  (not required but harmless)
  Cameras            (if you have camera-type nodes)
  Punctual Lights    (REQUIRED for light-type entities)

Transform:
  +Y Up

Geometry:
  Apply Modifiers
  Normals
  (disable Tangents, Vertex Colors)

Animation:
  Animations (if objects have baked animations)

Save as: world_dynamic.glb
```

**Important:** The `Punctual Lights` checkbox must be ON for light entities to be detected. Without it, lights export as empties and the parser will classify them as `empty` type.

---

## Server vs Client

| Environment | Loads Dynamic | Triggers | State Actions | Visual Actions | Light Effects | Mesh Tweens | Debug |
|-------------|--------------|----------|---------------|----------------|---------------|-------------|-------|
| Server | Yes | Yes | Yes | No | No | No | No |
| Client | Yes | No | No | Yes | Yes | Yes | No |
| World Viewer | Yes | No | No | No | Yes | Yes | Yes |

The server is fully authoritative over dynamic object state. State changes are replicated to clients via Lightyear. Clients only execute visual effects (light effects, mesh tweens, animations, text, sound).

---

## Components

| Component | Replicated | Description |
|-----------|-----------|-------------|
| `DynamicObject` | Yes | Marker with `object_type`, `object_id`, and `entity_type` (mesh/light/empty/camera) |
| `DynamicState` | Yes | Current state string and togglable flag |
| `DynamicBehavior` | No | Parsed trigger/action definitions (server + viewer only) |
| `InteractionRadius` | No | Proximity radius for interact triggers |
| `DynamicEnabled` | No | Whether the object processes triggers |
| `DynamicTimer` | No | Repeating timer for `onTimer` triggers |
| `DynamicDelay` | No | One-shot delay for `onDelay` triggers |
| `ActiveLightEffects` | No | Currently running intensity and color effects (light entities only) |
| `DynamicTween` | No | Active translation/rotation/scale tween |

---

## Cross-Object Targeting

Actions can reference other dynamic objects by name using the `target` parameter. The `DynamicObjectRegistry` maps node names to entities at load time.

Example: a switch that enables a door and starts a light in another part of the map:

```json
{
  "nodes": {
    "switch_01": {
      "type": "mesh",
      "triggers": [
        {
          "type": "playerOnInteract",
          "radius": 1.5,
          "actions": [
            { "type": "enable", "target": "door_secret" },
            { "type": "toggle_state" }
          ]
        }
      ],
      "state": { "initial": "off", "toggle": true }
    },
    "door_secret_light": {
      "type": "light",
      "light_info": { "light_type": "point", "color": [1.0, 1.0, 1.0], "intensity": 0.0 },
      "triggers": [
        {
          "type": "onTargetStateChange",
          "target": "switch_01",
          "to": "on",
          "actions": [
            { "type": "start_light_effect", "effect": "fixed", "intensity": 500.0 }
          ]
        }
      ]
    }
  }
}
```

---

## Debug Visualization

The world viewer includes debug visualization for dynamic objects.

**Toggle:** Press `D` to show/hide dynamic object debug meshes.

**Color:** Cyan (30% opacity) by default, configurable in `game_core_config.json` under `debug_colors.dynamic_object`.

Debug meshes are semi-transparent overlays rendered on top of dynamic object colliders. Only available in the world viewer (`DynamicPluginConfig::viewer()`).

**Key files:**

- `crates/game-core/src/dynamic/debug.rs` -- `DynamicDebugSettings`, toggle and visibility systems
- `crates/game-core/src/dynamic/processor.rs` -- spawns debug meshes alongside dynamic entities

---

## Best Practices

### Object Design

- Keep meshes simple -- the processor creates sensor colliders from the mesh geometry
- Use meaningful node names for cross-object targeting
- One trigger can have multiple actions -- compose effects instead of duplicating triggers
- Use `onEntitySpawn` for initialization (light effects, starting animations)
- Use `onStateChange` to decouple state transitions from visual effects

### Light Design

- Use `light_info` from the GLB to set initial light properties, then use effects for runtime behavior
- Combine flicker intensity with fixed color for torches
- Combine pulse intensity with cycle colors for disco/party lights
- Use `onTargetStateChange` for lights that react to remote switches
- Keep intensity values consistent with your scene's lighting scale

### State Management

- Use `toggle_state` for binary states (open/closed, on/off)
- Use `set_state` for multi-state objects (idle/warning/alarm/critical)
- Set `"toggle": true` in state config for togglable objects
- State is replicated -- clients react to changes via `sync_dynamic_state_visuals`
- Use `onStateChange` + `onTargetStateChange` for chaining instead of putting all logic in one trigger

### Performance

- Sensor colliders are trimesh-based -- keep geometry low-poly for trigger volumes
- Use `enable`/`disable` actions to deactivate objects that aren't needed
- Cross-object references resolve at load time via the registry
- Light effects and mesh tweens are ticked every frame -- keep the number of active effects reasonable

### Alignment

- Apply all transforms before export: select all, `Ctrl+A`, All Transforms
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
| Light effects not starting | Ensure the entity is type `light` with `light_info`, check that `ActiveLightEffects` component was spawned |
| Light not detected as light type | Ensure "Punctual Lights" is checked in Blender export settings |
| Tween not animating | Check that `offset`/`rotation`/`scale` param is a 3-element array, verify entity has a `Transform` component |
| Enter/exit not detecting | Verify the mesh has volume (not a single plane), check collider is created in logs |
| Parser not finding nodes | Ensure GLB is valid, run `cargo run -p glb-parser -- path/to/file.glb` to check output |
| onStateChange not firing | Verify the entity has state config and the `to` param matches the new state value |
| onTargetStateChange not firing | Verify the `target` param matches the other entity's `object_id` exactly |
