# game-camera

Flexible camera system with multiple view modes. Used by client apps and the world viewer.

## View Modes

| Mode | Use Case | Key Settings |
|------|----------|-------------|
| **FirstPerson** | Default player view | Sensitivity: 0.002, attached to player entity |
| **ThirdPerson** | Follow-cam behind player | Distance: 5.0, height: 2.0, smoothing: 0.1 |
| **FreeView** | Spectator/debug noclip | Speed: 10.0, no attachment |

Pitch is clamped to ~88 degrees (1.54 radians) in all modes.

## Key Types

- **`CameraPlugin`** — Main plugin. Reads mouse motion, updates yaw/pitch, applies transform rotation.
- **`GameCamera`** — Component storing yaw/pitch state. Provides `forward_direction()` and `right_direction()` helpers.
- **`CameraConfig`** — Runtime resource with current mode and all settings.
- **`GameCameraFileConfig`** — Deserialized from `assets/config/game_camera_config.json`. Contains presets for each mode.

## Presets

Construct via factory methods:
- `CameraConfig::client()` — First-person with standard settings
- `CameraConfig::viewer()` — Free view for world inspection
- `CameraConfig::server()` — Disabled (zero sensitivity/speed)
- `CameraConfig::first_person()`, `::third_person()`, `::free_view()` — Individual mode constructors

## Features

- `client` (default) — Full rendering and camera functionality
- `server` — Headless mode, no rendering dependencies
