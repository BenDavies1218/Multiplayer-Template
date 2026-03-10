# game-client

Client-side game logic: input handling, prediction, rendering, and network transport.

## Modules

| Module | Purpose |
|--------|---------|
| `app` | Builds the client Bevy app with window, rendering, and input plugins |
| `client` | `ClientPlugin` — input processing, predicted movement, crouch state |
| `client_config` | `GameClientConfig` loaded from `assets/config/game_client_config.json` |
| `renderer` | `FirstPersonPlugin` — skybox, character/projectile cosmetics, frame interpolation |
| `transport` | Network transport setup: UDP, WebSocket, WebTransport (runtime-configurable) |

## Plugins

### `ClientPlugin`
- Syncs mouse input to `CameraOrientation` as character action input
- Applies predicted movement via shared `apply_character_movement()`
- Sets up `InputMap` with keyboard/gamepad bindings for new characters
- Manages crouch state prediction

### `FirstPersonPlugin`
- Attaches camera to character at eye height
- Loads player models from catalog, attaches GLB scenes to character entities (falls back to capsule if model missing)
- Renders projectiles with sphere meshes
- Loads EXR skybox cubemap
- Runs frame interpolation for smooth rendering between network ticks

## Configuration

`GameClientConfig` contains:
- **Window** — Title, width, height
- **Input** — Key/gamepad bindings for all actions, cursor grab/release keys
- **Rendering** — Camera start position, eye height, projectile radius, interpolation ratio
- **Player** — Model catalog (map of ID to GLB path), selected model for this client
- **Transport** — Token expiration settings

## Dependencies

Depends on `game-core` (shared logic) and `game-camera` (camera system). Uses Lightyear client features with WebTransport, WebSocket, and UDP support.
