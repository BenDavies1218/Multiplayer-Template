# game-client

Client-side game logic: input handling, prediction, rendering, and network transport.

## Modules

| Module | Purpose |
|--------|---------|
| `app` | Builds the client Bevy app with window, rendering, and input plugins |
| `client` | `ClientPlugin` — input processing, predicted movement, crouch state |
| `client_config` | `GameClientConfig` loaded from `assets/config/game_client_config.json` |
| `character_rendering` | `CharacterRenderingPlugin` — preloads character models, attaches POV (local) and third-person (remote) models |
| `renderer` | `FirstPersonPlugin` — skybox, projectile cosmetics, camera follow, frame interpolation |
| `transport` | Network transport setup: UDP, WebSocket, WebTransport (runtime-configurable) |

## Plugins

### `ClientPlugin`
- Syncs mouse input to `CameraOrientation` as character action input
- Applies predicted movement via shared `apply_character_movement()`
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

## Configuration

`GameClientConfig` contains:
- **Window** — Title, width, height
- **Input** — Key/gamepad bindings for all actions, cursor grab/release keys
- **Rendering** — Camera start position, eye height, projectile radius, interpolation ratio
- **Character** — Model catalog (map of ID to `CharacterModelSet` with player/pov_empty/pov_weapons paths), selected model
- **Transport** — Token expiration settings

## Dependencies

Depends on `game-core` (shared logic) and `game-camera` (camera system). Uses Lightyear client features with WebTransport, WebSocket, and UDP support.
