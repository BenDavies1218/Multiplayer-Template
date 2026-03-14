# game-client

Client-side game logic: input handling, prediction, rendering, and network transport.

## Modules

| Module | Purpose |
|--------|---------|
| `app` | Builds the client Bevy app with window, rendering, and input plugins |
| `client` | `ClientPlugin` — input processing, predicted movement, crouch state |
| `client_config` | `GameClientConfig` loaded from `assets/config/game_client_config.json` |
| `character` | Attaches `InputMap` and physics bundle to new predicted characters |
| `character_rendering` | `CharacterRenderingPlugin` — preloads character models, attaches POV (local) and third-person (remote) models |
| `renderer` | `FirstPersonPlugin` — skybox, projectile cosmetics, camera follow, frame interpolation |
| `transport` | Network transport setup: UDP, WebSocket, WebTransport (runtime-configurable) |
| `prediction` | Prediction speed scaling for rollback thresholds |
| `movement` | Client-side movement: applies shared movement on predicted entities, syncs camera to ActionState |
| `input_device` | Runtime input device detection and hot-switching (keyboard/gamepad) |
| `diagnostics` | Predicted entity lifecycle logging |
| `dynamic_rendering` | Client-side visual action execution for dynamic objects (light effects start/stop, mesh tweens move_to/rotate_to/scale_to, animations, text, sound) |

## Plugins

### `ClientPlugin`
- Syncs mouse input to `CameraOrientation` as character action input
- Applies predicted movement via shared `apply_character_movement()` from `game-networking`
- Sets up `InputMap` with keyboard/gamepad bindings for new characters
- Manages crouch state prediction

### `FirstPersonPlugin`
- Attaches camera to character at eye height
- Includes `CharacterRenderingPlugin` for model attachment
- Renders projectiles with sphere meshes
- Loads EXR skybox cubemap
- Runs frame interpolation for smooth rendering between network ticks

### `CharacterRenderingPlugin`
- Preloads character models from config catalog (third-person + POV variants)
- Local player: attaches POV model as child of camera
- Remote players: attaches third-person model to entity (falls back to capsule if missing)

### `DynamicRenderingPlugin`

- Executes client-side visual actions from `DynamicActionEvent`: light effects (start/stop flicker, pulse, cycle, fixed), mesh tweens (move_to, rotate_to, scale_to with easing), legacy light intensity/color, animations, text, sound
- Reacts to replicated `DynamicState` changes from the server for visual state sync

## Configuration

`GameClientConfig` contains:
- **Window** — Title, width, height
- **Input** — Key/gamepad bindings for all actions, cursor grab/release keys
- **Rendering** — Camera start position, eye height, projectile radius, interpolation ratio
- **Character** — Model catalog (map of ID to `CharacterModelSet` with player/pov_empty/pov_weapons paths), selected model
- **Transport** — Token expiration settings

## Dependencies

Depends on `game-core` (config, world), `game-networking` (protocol, movement, replication types), and `game-camera` (camera system). Uses Lightyear client features with WebTransport, WebSocket, and UDP support.
