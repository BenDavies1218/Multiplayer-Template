# Client Config

**File:** `assets/config/game_client_config.json`
**Struct:** `GameClientConfig` in `crates/game-client/src/client_config.rs`

Used by the native and web clients. Controls window, input bindings, rendering, camera, transport, and debug visualization.

---

## Connection

How the client connects to the server.

```json
{
  "connection": {
    "server_host": "127.0.0.1",
    "server_port": 5888,
    "client_port": 0
  }
}
```

| Key           | Type   | Default       | Description                                           |
| ------------- | ------ | ------------- | ----------------------------------------------------- |
| `server_host` | string | `"127.0.0.1"` | Server address to connect to                          |
| `server_port` | u16    | `5888`        | Server port                                           |
| `client_port` | u16    | `0`           | Outbound client port (`0` = OS assigns automatically) |

---

## Window

```json
{
  "window": {
    "title": "Game Example",
    "width": 1024,
    "height": 768
  }
}
```

| Key      | Type   | Default               | Description             |
| -------- | ------ | --------------------- | ----------------------- |
| `title`  | string | `"Lightyear Example"` | Window title bar text   |
| `width`  | u32    | `1024`                | Window width in pixels  |
| `height` | u32    | `768`                 | Window height in pixels |

---

## Input

```json
{
  "input": {
    "active_device": "auto",
    "cursor_grab_button": "Left",
    "cursor_release_key": "Escape",
    "keyboard": { ... },
    "gamepad": { ... }
  }
}
```

| Key                  | Type   | Default    | Description                                                   |
| -------------------- | ------ | ---------- | ------------------------------------------------------------- |
| `active_device`      | string | `"auto"`   | `"auto"`, `"keyboard_mouse"`, or `"gamepad"`                  |
| `cursor_grab_button` | string | `"Left"`   | Mouse button to grab cursor (`"Left"`, `"Right"`, `"Middle"`) |
| `cursor_release_key` | string | `"Escape"` | Key to release grabbed cursor                                 |

### Keyboard Bindings

All values are Bevy `KeyCode` strings (e.g., `"KeyW"`, `"Space"`, `"ShiftLeft"`).

```json
{
  "input": {
    "keyboard": {
      "move_up": "KeyW",
      "move_down": "KeyS",
      "move_left": "KeyA",
      "move_right": "KeyD",
      "sprint": "ShiftLeft",
      "crouch": "ControlLeft",
      "prone": "KeyC",
      "jump": "Space",
      "mount_ledge": "Space",
      "fire": "",
      "aim_down_sights": "",
      "reload": "KeyR",
      "primary_weapon": "Digit1",
      "secondary_weapon": "Digit2",
      "interact": "KeyE",
      "lethal_equipment": "KeyQ",
      "tactical_equipment": "",
      "melee": "KeyV",
      "weapon_inspect": "KeyG",
      "armor_plate": "KeyF",
      "alternate_fire": "KeyB",
      "killstreak1": "Digit3",
      "killstreak2": "Digit4",
      "killstreak3": "Digit5",
      "field_upgrade": "KeyX",
      "text_chat": "KeyT",
      "team_chat": "KeyY",
      "ping": "KeyZ",
      "push_to_talk": "CapsLock",
      "gesture1": "F1",
      "gesture2": "F2",
      "gesture3": "F3",
      "gesture4": "F4",
      "scoreboard": "Tab",
      "map": "KeyM",
      "inventory": "KeyI",
      "pause": "Escape",
      "night_vision": "KeyN"
    }
  }
}
```

Empty strings (`""`) mean the action is unbound.

### Gamepad Bindings

Values are Bevy `GamepadButton` or `GamepadStick` strings.

```json
{
  "input": {
    "gamepad": {
      "move_stick": "LeftStick",
      "look_stick": "RightStick",
      "sprint": "LeftThumb",
      "crouch": "RightThumb",
      "jump": "South",
      "fire": "RightTrigger",
      "aim_down_sights": "LeftTrigger",
      "reload": "West",
      "switch_weapon": "North",
      "lethal_equipment": "LeftTrigger2",
      "tactical_equipment": "RightTrigger2",
      "melee": "East",
      "killstreak1": "DPadRight",
      "killstreak2": "DPadRight",
      "killstreak3": "DPadRight",
      "field_upgrade": "DPadLeft",
      "ping": "DPadLeft",
      "armor_plate": "DPadUp",
      "night_vision": "DPadDown",
      "scoreboard": "Select",
      "pause": "Start"
    }
  }
}
```

---

## Rendering

```json
{
  "rendering": {
    "camera_start_position": [0.0, 2.0, 0.0],
    "eye_height_offset": 0.5,
    "projectile_radius": 1.0,
    "interpolation_send_ratio": 2.0
  }
}
```

| Key                        | Type     | Default           | Description                                                |
| -------------------------- | -------- | ----------------- | ---------------------------------------------------------- |
| `camera_start_position`    | [f32; 3] | `[0.0, 2.0, 0.0]` | Initial camera world position                              |
| `eye_height_offset`        | f32      | `0.5`             | Eye offset above capsule center (first-person view height) |
| `projectile_radius`        | f32      | `1.0`             | Visual radius of projectiles                               |
| `interpolation_send_ratio` | f32      | `2.0`             | Multiplier for interpolation buffer timing                 |

---

## Camera

Camera mode presets. Each mode (first-person, third-person, free-view) has independent settings.

```json
{
  "camera": {
    "pitch_clamp_radians": 1.5,
    "start_position": [0.0, 5.0, 10.0],
    "first_person": {
      "sensitivity": 0.002,
      "free_camera_speed": 0.0,
      "third_person_distance": 0.0,
      "third_person_height": 0.0,
      "smooth_camera": false,
      "smooth_factor": 0.1
    },
    "third_person": {
      "sensitivity": 0.002,
      "free_camera_speed": 0.0,
      "third_person_distance": 5.0,
      "third_person_height": 2.0,
      "smooth_camera": true,
      "smooth_factor": 0.1
    },
    "free_view": {
      "sensitivity": 0.002,
      "free_camera_speed": 10.0,
      "third_person_distance": 0.0,
      "third_person_height": 0.0,
      "smooth_camera": false,
      "smooth_factor": 0.1
    }
  }
}
```

| Key                   | Type     | Default            | Description                                    |
| --------------------- | -------- | ------------------ | ---------------------------------------------- |
| `pitch_clamp_radians` | f32      | `1.5`              | Max vertical look angle (radians, ~86 degrees) |
| `start_position`      | [f32; 3] | `[0.0, 5.0, 10.0]` | Initial camera position before player spawns   |

**Per-preset fields:**

| Key                     | Type | Description                                      |
| ----------------------- | ---- | ------------------------------------------------ |
| `sensitivity`           | f32  | Mouse/stick look sensitivity                     |
| `free_camera_speed`     | f32  | Movement speed in free-view mode (0 = disabled)  |
| `third_person_distance` | f32  | Camera distance behind player (0 = first-person) |
| `third_person_height`   | f32  | Camera height offset above player                |
| `smooth_camera`         | bool | Enable smooth camera interpolation               |
| `smooth_factor`         | f32  | Smoothing strength (lower = smoother, more lag)  |

---

## Character

Character model selection.

```json
{
  "character": {
    "selected_model": "default",
    "model_catalog": {
      "default": {
        "player": "models/characters/default/player.glb",
        "pov_empty": "models/characters/default/pov_empty.glb",
        "pov_weapons": {}
      }
    }
  }
}
```

| Key              | Type   | Default         | Description                             |
| ---------------- | ------ | --------------- | --------------------------------------- |
| `selected_model` | string | `"default"`     | Which model set to use from the catalog |
| `model_catalog`  | map    | 1 default entry | Named model sets                        |

**Per model set:**

| Key           | Type   | Description                                      |
| ------------- | ------ | ------------------------------------------------ |
| `player`      | string | Third-person model path (what other players see) |
| `pov_empty`   | string | First-person empty hands model                   |
| `pov_weapons` | map    | First-person weapon models, keyed by weapon name |

---

## Transport

```json
{
  "transport": {
    "token_expiration": -1,
    "simulate_latency": false
  }
}
```

| Key                | Type | Default | Description                                                           |
| ------------------ | ---- | ------- | --------------------------------------------------------------------- |
| `token_expiration` | i32  | `-1`    | Connection token expiration time (`-1` = never expires)               |
| `simulate_latency` | bool | `false` | Enable Lightyear's link conditioner (~100ms jitter) for local testing |

---

## Debug

Debug visualization colors (RGBA) and toggle keys.

```json
{
  "debug": {
    "colors": {
      "collision": [1.0, 0.0, 0.0, 0.3],
      "death_zone": [1.0, 0.0, 0.0, 0.3],
      "damage_zone": [1.0, 1.0, 0.0, 0.3],
      "trigger_zone": [0.0, 0.5, 1.0, 0.3],
      "spawn_point": [0.0, 1.0, 0.0, 0.3],
      "dynamic_object": [0.0, 1.0, 1.0, 0.3]
    },
    "toggle_keys": {
      "collision": "KeyC",
      "zone": "KeyZ",
      "dynamic": "KeyD"
    }
  }
}
```

| Color Key        | Default RGBA           | Visual              |
| ---------------- | ---------------------- | ------------------- |
| `collision`      | `[1.0, 0.0, 0.0, 0.3]` | Red, 30% opacity    |
| `death_zone`     | `[1.0, 0.0, 0.0, 0.3]` | Red, 30% opacity    |
| `damage_zone`    | `[1.0, 1.0, 0.0, 0.3]` | Yellow, 30% opacity |
| `trigger_zone`   | `[0.0, 0.5, 1.0, 0.3]` | Cyan, 30% opacity   |
| `spawn_point`    | `[0.0, 1.0, 0.0, 0.3]` | Green, 30% opacity  |
| `dynamic_object` | `[0.0, 1.0, 1.0, 0.3]` | Cyan, 30% opacity   |

| Toggle Key  | Default | What It Shows                                   |
| ----------- | ------- | ----------------------------------------------- |
| `collision` | `KeyC`  | Collision mesh wireframes                       |
| `zone`      | `KeyZ`  | Zone boundaries (death, damage, trigger, spawn) |
| `dynamic`   | `KeyD`  | Dynamic object boundaries                       |

---

## Diagnostics

```json
{
  "enable_diagnostics": true
}
```

| Key                  | Type | Default | Description                        |
| -------------------- | ---- | ------- | ---------------------------------- |
| `enable_diagnostics` | bool | `false` | Enable runtime diagnostics overlay |
