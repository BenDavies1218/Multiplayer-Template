# Native Client

The native desktop client for the multiplayer Bevy game. This binary provides a full 3D client with rendering, input handling, client-side prediction, and smooth interpolation.

## Purpose

The native client provides:

- 3D rendering with Bevy's PBR renderer
- Client-side prediction for local player
- Interpolation for remote players
- Input handling with leafwing-input-manager
- Physics simulation synchronized with the server
- Flexible camera system (first-person, third-person, free-view)

## Features

- **Client-Side Prediction**: Immediate local response without network delay
- **100ms Interpolation Buffer**: Industry-standard smoothing for remote players
- **Rollback Correction**: Visual error smoothing when server disagrees
- **Frame Interpolation**: Smooth rendering between 64Hz physics ticks
- **3D Graphics**: Full PBR rendering with lighting and shadows
- **Camera Modes**: First-person, third-person, and free-view cameras
- **Input Handling**: Keyboard and gamepad support

## Platform Support

- Linux (X11)
- macOS
- Windows

## Building

### Development Build

```bash
cargo build -p native
```

### Release Build (Performance Build)

```bash
cargo build -p native --release
```

The first build will take several minutes as it compiles Bevy and dependencies.

## Running

### Basic Usage

```bash
cargo run -p native -- client -c 1
```

The `-c` (or `--client-id`) flag is required and must be unique for each client.

### Fast Development Build

```bash
cargo dev-native
```

This uses dynamic linking for faster incremental builds and defaults to client ID 1.

### Run Multiple Clients

```bash
# Terminal 1
cargo run -p native -- client -c 1

# Terminal 2
cargo run -p native -- client -c 2

# Terminal 3
cargo run -p native -- client -c 3
```

## Controls

### Keyboard

- **W/A/S/D**: Move forward/left/backward/right
- **Space**: Jump
- **Left Shift**: Sprint
- **C**: Crouch
- **Q**: Shoot projectile
- **Left Click**: Grab/lock cursor
- **Escape**: Release cursor

### Gamepad

- **Left Stick**: Move
- **South Button (A/Cross)**: Jump
- **Left Thumb**: Sprint
- **East Button (B/Circle)**: Crouch

## Client Architecture

### Dependencies

The native client depends on:

- **game-core**: Shared protocol, game logic, configuration, and dynamic objects (`DynamicPlugin`)
- **game-client**: Client-specific systems (prediction, rendering, input, transport, `DynamicRenderingPlugin`)
- **game-camera**: Camera system with multiple view modes

### Configuration

All settings are loaded from JSON config files at startup:

- `assets/config/game_core_config.json` — Networking, movement, physics
- `assets/config/game_client_config.json` — Window, input bindings, rendering
- `assets/config/game_camera_config.json` — Camera modes and sensitivity

## Network Configuration

### Transport Protocol

The native client uses **UDP** by default. Transport is configured in the client transport code (`crates/game-client/src/transport.rs`). Available transports:

- **UDP** (native only, default)
- **WebTransport**
- **WebSocket**

### Server Connection

The server address and port are configured in `assets/config/game_core_config.json`:

```json
{
  "networking": {
    "server_host": "127.0.0.1",
    "server_port": 5888
  }
}
```

### Interpolation Settings

Interpolation is configured via JSON config:

- **Interpolation buffer**: 100ms (set in `game_core_config.json` → `networking.interpolation_buffer_ms`)
- **Send interval ratio**: 2.0 (set in `game_client_config.json` → `rendering.interpolation_send_ratio`)

**100ms** is the industry standard for smooth remote player interpolation.

## Rendering Configuration

### Window Settings

Default window: **1024x768**

Configured in `assets/config/game_client_config.json`:

```json
{
  "window": {
    "title": "Lightyear Example",
    "width": 1024,
    "height": 768
  }
}
```

### Camera Modes

The camera system supports three modes, configured in `assets/config/game_camera_config.json`:

- **First Person**: Direct control, sensitivity 0.002
- **Third Person**: Distance 5.0, height 2.0, smooth camera enabled
- **Free View**: Spectator/debug camera, speed 10.0

## Performance

### Frame Rate

- **Physics**: 64Hz fixed timestep
- **Rendering**: Variable, vsync-limited (typically 60 FPS)
- **Network Send**: 64Hz (every 15.625ms)

### Optimization Tips

1. **Use release builds** for significantly better performance:

   ```bash
   cargo run -p native --release -- client -c 1
   ```

2. **Use dev alias** for faster incremental builds during development:

   ```bash
   cargo dev-native
   ```

## Troubleshooting

### Connection Failed

If the client can't connect:

1. Ensure the server is running:

   ```bash
   cargo run -p server -- server
   ```

2. Check server address in `assets/config/game_core_config.json`

3. Verify firewall allows connections to port 5888

### Stuttering/Lag

If you see visual stuttering:

1. **Increase interpolation delay**: Change `interpolation_buffer_ms` from 100 to 150 in `game_core_config.json`
2. **Use release build**: Debug builds are much slower
3. **Check frame rate**: Open browser console or use diagnostics

### Slow Compilation

First build takes 5-10 minutes. Subsequent builds are much faster.

Speed up builds:

- Use `cargo dev-native` for dynamic linking
- Install `lld` linker (Linux) or `zld` (macOS)
- Use `mold` linker for fastest linking (Linux)

## Related Documentation

- [Introduction](../introduction.md)
- [Server](server.md)
- [Web Client](web-client.md)
- [World Viewer](world-viewer.md)
