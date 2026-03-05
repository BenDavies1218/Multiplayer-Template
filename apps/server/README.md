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
  - UDP (fast, unreliable)
  - WebSocket (browser-compatible, TCP-based)
  - WebTransport (modern, QUIC-based with TLS)
- **Netcode Security**: Token-based authentication
- **Physics Simulation**: Avian3d running in FixedUpdate
- **Player Management**: Automatic spawning/despawning on connect/disconnect

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

Or with the built binary:

```bash
./target/release/server server
```

### Configuration

The server uses WebTransport by default on port **5000**.

To change transport protocols, edit `crates/game-core/src/common/cli.rs`:

```rust
// UDP
transport: ServerTransports::Udp {
    local_port: SERVER_PORT,
}

// WebSocket
transport: ServerTransports::WebSocket {
    local_port: SERVER_PORT,
}

// WebTransport (default)
transport: ServerTransports::WebTransport {
    local_port: SERVER_PORT,
    certificate: WebTransportCertificateSettings::FromFile {
        cert: "./certificates/cert.pem".to_string(),
        key: "./certificates/key.pem".to_string(),
    },
}
```

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

Run the server from the repository root so these paths resolve correctly.

## Server Architecture

### Dependencies

The server binary depends on:
- **game-core**: Shared protocol and game logic
- **game-server**: Server-specific systems and logic

### Key Systems

1. **handle_connected** (crates/game-server/src/server.rs:168)
   - Spawns player character when client connects
   - Assigns color and spawn position
   - Creates replicated entity

2. **handle_character_actions** (crates/game-server/src/server.rs:42)
   - Processes player input (movement, jumping)
   - Applies physics forces
   - Runs in FixedUpdate

3. **player_shoot** (crates/game-server/src/server.rs:70)
   - Handles shoot action
   - Spawns projectiles with replicate-once
   - Broadcasts to all clients

4. **despawn_system** (crates/game-server/src/server.rs:58)
   - Cleans up projectiles after lifetime expires

## Network Configuration

### Default Settings

- **Port**: 5000 (defined in `game-core/src/common/shared.rs`)
- **Protocol**: WebTransport
- **Send Interval**: 64Hz (every 15.625ms)
- **Fixed Timestep**: 64Hz
- **Client Timeout**: 3 seconds

### Modify Port

Edit `crates/game-core/src/common/shared.rs`:

```rust
pub const SERVER_PORT: u16 = 5000; // Change this
```

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

### Docker (Optional)

```dockerfile
FROM rust:1.93 as builder
WORKDIR /app
COPY . .
RUN cargo build -p server --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/server /usr/local/bin/
COPY certificates /certificates
CMD ["server", "server"]
```

## Monitoring

The server logs important events:

```
INFO server: Client connected with client-id 12345. Spawning character entity.
INFO server: Created entity Entity { index: 5, generation: 1 } for client 12345
```

## Troubleshooting

### Port Already in Use

If you see "address already in use":
```bash
# Find process using port 5000
lsof -i :5000

# Kill process (replace PID)
kill -9 <PID>
```

### Certificate Not Found

Ensure you're running from the repository root:
```bash
cd /path/to/Multiplayer-Template
cargo run -p server -- server
```

### Connection Timeouts

Check firewall settings allow incoming connections on port 5000:
```bash
# Linux (ufw)
sudo ufw allow 5000

# macOS
# System Settings > Network > Firewall > Options
```

## Related Documentation

- [Root README](../../README.md)
- [Native Client README](../native/README.md)
- [Web Client README](../web/README.md)
