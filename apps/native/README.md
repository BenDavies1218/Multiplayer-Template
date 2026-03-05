# Native Client

The native desktop client for the multiplayer Bevy game. This binary provides a full 3D client with rendering, input handling, client-side prediction, and smooth interpolation.

## Purpose

The native client provides:
- 3D rendering with Bevy's PBR renderer
- Client-side prediction for local player
- Interpolation for remote players
- Input handling with leafwing-input-manager
- Physics simulation synchronized with the server

## Features

- **Client-Side Prediction**: Immediate local response without network delay
- **100ms Interpolation Buffer**: Industry-standard smoothing for remote players
- **Rollback Correction**: Visual error smoothing when server disagrees
- **Frame Interpolation**: Smooth rendering between 64Hz physics ticks
- **3D Graphics**: Full PBR rendering with lighting and shadows
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

### Release Build (Recommended for Playing)

```bash
cargo build -p native --release
```

The first build will take several minutes as it compiles Bevy and dependencies.

## Running

### Basic Usage

```bash
cargo run -p native -- client --client-id 1
```

The `--client-id` (or `-c`) flag is required and must be unique for each client.

### Run Multiple Clients

```bash
# Terminal 1
cargo run -p native -- client -c 1

# Terminal 2
cargo run -p native -- client -c 2

# Terminal 3
cargo run -p native -- client -c 3
```

### Connect to Remote Server

Edit `crates/game-core/src/common/shared.rs`:

```rust
pub const SERVER_ADDR: SocketAddr = SocketAddr::new(
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), // Change to server IP
    SERVER_PORT
);
```

## Controls

### Keyboard

- **W/A/S/D**: Move forward/left/backward/right
- **Space**: Jump
- **Left Mouse Button**: Shoot projectile

### Gamepad

- **Left Stick**: Move
- **South Button (A/Cross)**: Jump
- **Right Trigger**: Shoot

## Client Architecture

### Dependencies

The native client depends on:
- **game-core**: Shared protocol and game logic
- **game-client**: Client-specific systems (prediction, rendering, input)

### Key Systems

1. **add_player_input** (crates/game-client/src/client.rs:38)
   - Sets up input map for local player
   - Binds keyboard/gamepad to actions

2. **add_character_cosmetics** (crates/game-client/src/renderer.rs:100)
   - Adds 3D mesh and material to characters
   - Uses player-specific colors

3. **add_visual_interpolation_components** (crates/game-client/src/renderer.rs:70)
   - Enables frame interpolation for smooth visuals
   - Added when predicted entities spawn

4. **disable_projectile_rollback** (crates/game-client/src/renderer.rs:181)
   - Optimizes projectile prediction
   - Disables rollback after initial spawn

## Network Configuration

### Transport Protocol

The client uses **WebTransport** by default. To change, edit `crates/game-core/src/common/cli.rs`:

```rust
// WebTransport (default, fastest)
transport: ClientTransports::WebTransport,

// WebSocket (browser-compatible)
transport: ClientTransports::WebSocket,

// UDP (not available in WASM)
transport: ClientTransports::Udp,
```

### Interpolation Settings

Configured in `crates/game-core/src/common/client.rs:62`:

```rust
InterpolationConfig::default()
    .with_send_interval_ratio(2.0)
    .with_min_delay(Duration::from_millis(100))  // 100ms buffer
```

**100ms** is the industry standard for smooth remote player interpolation.

### Input Delay

Input delay is set to **0** for the native client (apps/native/src/main.rs:45):

```rust
InputTimelineConfig::default()
    .with_input_delay(InputDelayConfig::fixed_input_delay(0))
```

This provides instant local response since we're using client-side prediction.

## Rendering Configuration

### Window Settings

Default window: **1024x768**, auto-vsync

Edit in `crates/game-core/src/common/cli.rs`:

```rust
#[cfg(feature = "gui")]
pub fn window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: format!("Lightyear Example: {}", env!("CARGO_PKG_NAME")),
            resolution: (1024, 768).into(),  // Change resolution
            present_mode: PresentMode::AutoVsync,
            ..Default::default()
        }),
        ..default()
    }
}
```

### Camera

The camera is positioned at `(0, 4.5, -9)` looking at the origin.

Edit in `crates/game-client/src/renderer.rs:52`:

```rust
commands.spawn((
    Camera3d::default(),
    Transform::from_xyz(0.0, 4.5, -9.0)
        .looking_at(Vec3::ZERO, Dir3::Y),
));
```

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

2. **Disable vsync** for higher frame rates (edit window settings):
   ```rust
   present_mode: PresentMode::Immediate,
   ```

3. **Reduce visual quality** by disabling shadows or lowering resolution

## Troubleshooting

### Connection Failed

If the client can't connect:

1. Ensure the server is running:
   ```bash
   cargo run -p server -- server
   ```

2. Check server address in `game-core/src/common/shared.rs`

3. Verify firewall allows connections to port 5000

### Certificate Error (WebTransport)

For self-signed certificates, the client needs the certificate digest.

Native clients automatically trust self-signed certificates in development. For production, use proper CA-signed certificates.

### Stuttering/Lag

If you see visual stuttering:

1. **Check network conditions**: Use the conditioner settings in `cli.rs`
2. **Increase interpolation delay**: Change from 100ms to 150ms
3. **Use release build**: Debug builds are much slower
4. **Check frame rate**: Press F12 to see diagnostics

### Slow Compilation

First build takes 5-10 minutes. Subsequent builds are much faster (under 30 seconds).

Speed up builds:
- Use `cargo build` (not `--release`) during development
- Install `lld` linker (Linux) or `zld` (macOS)
- Use `mold` linker for fastest linking (Linux)

## Advanced Configuration

### Custom Client ID

Instead of command-line argument, you can modify `cli()` function:

```rust
// In game-core/src/common/cli.rs
Cli {
    mode: Some(Mode::Client {
        client_id: Some(42),  // Hardcoded client ID
    })
}
```

### Network Conditioner

Simulate network conditions for testing (edit `cli.rs`):

```rust
let conditioner = LinkConditionerConfig {
    incoming_latency: Duration::from_millis(100),  // Add 100ms latency
    incoming_jitter: Duration::from_millis(20),    // Add jitter
    incoming_loss: 0.05,                           // 5% packet loss
};
```

## Development Tips

### Hot Reloading

Bevy supports hot reloading of assets. Place assets in `assets/` and they'll reload on change.

### Debugging

Enable debug rendering:

```rust
// Add to main.rs
app.add_plugins(bevy::gizmos::GizmoPlugin::default());
```

### Profiling

Use `cargo flamegraph` or Tracy profiler for performance analysis.

## Related Documentation

- [Root README](../../README.md)
- [Server README](../server/README.md)
- [Web Client README](../web/README.md)
