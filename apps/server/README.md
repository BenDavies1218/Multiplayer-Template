# Server Binary

The dedicated game server for the multiplayer Bevy game. This binary runs the authoritative game server that manages game state, player connections, and physics simulation.

## Purpose

The server is the authority for:

- Player positions and physics
- Game state and world entities
- Input validation and processing
- Network replication to all clients

## Features

- **Headless Mode**: Runs without graphics, optimized for server deployment
- **Multiple Transport Protocols**:
  - UDP (fast, default)
  - WebSocket (browser-compatible, TCP-based)
  - WebTransport (modern, QUIC-based with TLS)
- **Netcode Security**: Token-based authentication
- **Physics Simulation**: Avian3d running in FixedUpdate
- **Player Management**: Automatic spawning/despawning on connect/disconnect
- **Dynamic Objects**: Server-authoritative trigger detection and state management for interactable objects
- **JSON Configuration**: All settings driven by config files

## Building

### Development Build

```bash
cargo build -p server
```

### Release Build (Recommended for Production)

```bash
cargo build -p server --release
```

The binary will be in `target/release/server`.

## Running

### Basic Usage

```bash
cargo run -p server -- server
```

### Fast Development Build

```bash
cargo dev-server
```

This uses dynamic linking for faster incremental builds.

### With the Built Binary

```bash
./target/release/server server
```

## Configuration

All server settings are loaded from JSON config files in `assets/config/`:

### Network Settings (`game_core_config.json`)

```json
{
  "networking": {
    "server_host": "127.0.0.1",
    "server_port": 5888,
    "fixed_timestep_hz": 64.0,
    "send_interval_hz": 64.0,
    "client_timeout_secs": 3,
    "interpolation_buffer_ms": 100
  }
}
```

### Server Settings (`game_server_config.json`)

```json
{
  "projectile": {
    "lifetime_ms": 5000,
    "velocity": 10.0
  },
  "spawning": {
    "player_colors": ["limegreen", "pink", "yellow", "aqua", "crimson", "gold"]
  },
  "transport": {
    "transport_type": "udp",
    "certificate_sans": ["localhost", "127.0.0.1", "::1"]
  }
}
```

### Changing Transport Protocol

Edit `transport_type` in `assets/config/game_server_config.json`:

- `"udp"` — UDP (default, fastest)
- `"websocket"` — WebSocket (browser-compatible)
- `"webtransport"` — WebTransport (QUIC-based, requires TLS certificates)

### Changing Port

Edit `server_port` in `assets/config/game_core_config.json`. Default is **5888**.

## WebTransport Certificates

WebTransport requires TLS certificates. For development, you can generate self-signed certificates:

### Generate Certificates

```bash
# Create certificates directory if it doesn't exist
mkdir -p certificates

# Generate self-signed certificate (requires openssl)
openssl req -x509 -newkey rsa:4096 -keyout certificates/key.pem \
    -out certificates/cert.pem -sha256 -days 365 -nodes \
    -subj "/CN=localhost"

# Generate certificate digest for web clients
openssl x509 -in certificates/cert.pem -outform der | \
    openssl dgst -sha256 -binary | base64 > certificates/digest.txt
```

### Certificate Paths

The server expects certificates at:

- `./certificates/cert.pem`
- `./certificates/key.pem`

The server writes the certificate digest to `./certificates/digest.txt` automatically when using self-signed certificates.

Run the server from the repository root so these paths resolve correctly.

## Server Architecture

### Dependencies

The server binary depends on:

- **game-core**: Shared protocol, game logic, configuration, and dynamic objects (`DynamicPlugin`)
- **game-server**: Server-specific systems, transport, and logic

### Key Systems

1. **handle_connected** — Spawns player character when client connects, assigns color and spawn position
2. **handle_character_actions** — Processes player input (movement, jumping), applies physics forces in FixedUpdate
3. **player_shoot** — Handles shoot action, spawns projectiles with replicate-once
4. **despawn_system** — Cleans up projectiles after lifetime expires

## Performance

### Optimization Settings

The server is built with optimized dependencies even in dev mode:

```toml
[profile.dev.package."*"]
opt-level = 3  # Optimize dependencies

[profile.dev]
opt-level = 1  # Fast incremental builds
```

### Release Profile

```toml
[profile.release]
lto = true              # Link-time optimization
opt-level = 3           # Maximum optimization
codegen-units = 1       # Better optimization
incremental = false     # Consistent builds
```

## Deployment

### Running in Production

```bash
# Build release binary
cargo build -p server --release

# Run server
./target/release/server server
```

### System Requirements

- **Minimum**: 1 CPU core, 512MB RAM
- **Recommended**: 2+ CPU cores, 1GB+ RAM for 10+ players

### Docker

```bash
docker build -f Dockerfile.server -t multiplayer-server .
docker run -p 5888:5888 -v ./certificates:/certificates multiplayer-server
```

The Docker image uses `rust:1.93-bookworm` for building and `debian:bookworm-slim` for the runtime.

## Monitoring

The server logs important events:

```text
INFO server: Client connected with client-id 12345. Spawning character entity.
INFO server: Created entity Entity { index: 5, generation: 1 } for client 12345
```

## Troubleshooting

### Port Already in Use

If you see "address already in use":

```bash
# Find process using port 5888
lsof -i :5888

# Kill process (replace PID)
kill -9 <PID>
```

### Certificate Not Found

Ensure you're running from the repository root:

```bash
cd /path/to/multiplayer-bevy
cargo run -p server -- server
```

### Connection Timeouts

Check firewall settings allow incoming connections on port 5888:

```bash
# Linux (ufw)
sudo ufw allow 5888

# macOS
# System Settings > Network > Firewall > Options
```

## Related Documentation

- [Root README](../../README.md)
- [Native Client README](../native/README.md)
- [Web Client README](../web/README.md)
- [World Viewer README](../world-viewer/README.md)
