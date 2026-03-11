# game-server

Server-side authoritative game logic and player management. Runs headless (no rendering).

## Modules

| Module | Purpose |
|--------|---------|
| `app` | Builds the server Bevy app in headless mode |
| `server` | `ServerPlugin` — authoritative movement, shooting, entity lifecycle |
| `server_config` | `GameServerConfig` loaded from `assets/config/game_server_config.json` |
| `transport` | Network transport setup and TLS certificate handling |

## Plugin: `ServerPlugin`

### Systems
- **`handle_character_actions()`** — Reads replicated input and applies authoritative movement via shared `apply_character_movement()`

### Observers
- **`handle_new_client()`** — Triggered on new client connection
- **`handle_connected()`** — Triggered when client completes handshake; spawns character entity with physics bundle, assigns color from palette

## Configuration

`GameServerConfig` contains:

- **Spawning** — Fallback spawn position (angle, radius, height), player color palette
- **Transport** — Type selection (`udp` / `webtransport` / `websocket`), certificate SANs

## Dependencies

Depends on `game-core` (shared logic). Uses Lightyear server features with WebTransport, WebSocket, and UDP support. Async support via `async-compat`.
