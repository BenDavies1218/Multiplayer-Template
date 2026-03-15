# game-client

Client-side game logic: input handling, prediction, rendering, and network transport.

## Sub-plugin Architecture

The two main convenience plugins, `ClientPlugin` and `FirstPersonPlugin`, are thin wrappers that compose smaller, independently-usable sub-plugins. Apps can replace a convenience plugin with a hand-picked subset of sub-plugins for debugging or special-purpose builds.

### `ClientPlugin` wraps:

| Sub-plugin | Purpose |
|------------|---------|
| `InputPlugin` | Input device detection, InputMap rebuilding, camera-to-ActionState sync |
| `PredictionPlugin` | Predicted movement simulation and prediction diagnostics |
| `LifecyclePlugin` | Attaches InputMap, physics bundle, and orientation to new predicted characters |

### `FirstPersonPlugin` wraps:

| Sub-plugin | Purpose |
|------------|---------|
| `CameraPlugin` | Camera modes from `game-camera` |
| `CharacterRenderingPlugin` | Character model preloading and attachment |
| `ClientSkyboxPlugin` | Skybox loading and camera spawning |
| `VisualInterpolationPlugin` | Frame interpolation for smooth rendering between network ticks |
| `ProjectileCosmeticsPlugin` | Projectile sphere meshes and physics colliders |
| `CursorPlugin` | Cursor grab/release handling |
| `AnimationPlugin` | Auto-play glTF animations |

`FirstPersonPlugin` also registers the `fps_camera_follow` system directly, as it is the glue between camera, character, and config.

## Modules

| Module | Purpose |
|--------|---------|
| `app` | Builds the client Bevy app with window, rendering, and input plugins |
| `client` | `ClientPlugin` — thin wrapper over InputPlugin + PredictionPlugin + LifecyclePlugin |
| `client_config` | `GameClientConfig` loaded from `assets/config/game_client_config.json` |
| `character` | Attaches `InputMap` and physics bundle to new predicted characters |
| `character_rendering` | `CharacterRenderingPlugin` — preloads character models, attaches POV (local) and third-person (remote) models |
| `renderer` | `FirstPersonPlugin` — thin wrapper over rendering sub-plugins plus camera follow |
| `transport` | Network transport setup: UDP, WebSocket, WebTransport (runtime-configurable) |
| `prediction` | Prediction speed scaling for rollback thresholds |
| `movement` | Client-side movement: applies shared movement on predicted entities, syncs camera to ActionState |
| `input_device` | Runtime input device detection and hot-switching (keyboard/gamepad) |
| `diagnostics` | Predicted entity lifecycle logging |
| `dynamic_rendering` | Client-side visual action execution for dynamic objects |
| `input_plugin` | `InputPlugin` sub-plugin |
| `prediction_plugin` | `PredictionPlugin` sub-plugin |
| `lifecycle_plugin` | `LifecyclePlugin` sub-plugin |
| `visual_interpolation_plugin` | `VisualInterpolationPlugin` sub-plugin |
| `projectile_cosmetics_plugin` | `ProjectileCosmeticsPlugin` sub-plugin |
| `cursor_plugin` | `CursorPlugin` sub-plugin |
| `client_skybox_plugin` | `ClientSkyboxPlugin` sub-plugin |
| `animation_plugin` | `AnimationPlugin` sub-plugin |

## Plugins

### `ClientPlugin`
Convenience wrapper that adds `InputPlugin`, `PredictionPlugin`, and `LifecyclePlugin`.

### `FirstPersonPlugin`
Convenience wrapper that adds `CameraPlugin`, `CharacterRenderingPlugin`, `ClientSkyboxPlugin`, `VisualInterpolationPlugin`, `ProjectileCosmeticsPlugin`, `CursorPlugin`, `AnimationPlugin`, and the `fps_camera_follow` system.

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

Depends on `game-core` (config, world), `game-protocol` (shared types), `game-networking` (protocol registration, movement, replication types), and `game-camera` (camera system). Uses Lightyear client features with WebTransport, WebSocket, and UDP support.
