# Server Config

**File:** `assets/config/game_server_config.json`
**Struct:** `GameServerConfig` in `crates/game-server/src/server_config.rs`

Used by the headless authoritative server. Controls bind address, player spawning, transport protocol, and diagnostics.

---

## Connection

```json
{
  "connection": {
    "server_host": "127.0.0.1",
    "server_port": 5888,
    "steam_app_id": 480
  }
}
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `server_host` | string | `"127.0.0.1"` | Address to bind the server to (`"0.0.0.0"` for all interfaces) |
| `server_port` | u16 | `5888` | Port to listen on |
| `steam_app_id` | u32 | `480` | Steam application ID for authentication (`480` = Spacewar test app) |

---

## Spawning

Controls where and how players spawn when they connect.

```json
{
  "spawning": {
    "fallback_angle_multiplier": 5.0,
    "fallback_radius": 2.0,
    "fallback_height": 3.0,
    "player_colors": [
      "limegreen", "pink", "yellow", "cyan",
      "orange", "purple", "red", "blue",
      "white", "magenta", "gold", "tomato"
    ]
  }
}
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `fallback_angle_multiplier` | f32 | `5.0` | Angle spread between fallback spawn positions (degrees × player index) |
| `fallback_radius` | f32 | `2.0` | Radius from origin for fallback spawn circle |
| `fallback_height` | f32 | `3.0` | Y-axis offset for fallback spawns (prevents spawning inside ground) |
| `player_colors` | [string] | 12 CSS colors | Color names assigned to players in order of connection |

Fallback spawns are used when no `spawn_` prefixed objects exist in the zones GLB. Players are placed in a circle around the origin using the angle multiplier and radius.

---

## Transport

```json
{
  "transport": {
    "transport_type": "udp",
    "certificate_sans": ["localhost", "127.0.0.1", "::1"]
  }
}
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `transport_type` | string | `"udp"` | Transport protocol: `"udp"`, `"websocket"`, or `"webtransport"` |
| `certificate_sans` | [string] | `["localhost", "127.0.0.1", "::1"]` | Subject Alternative Names for self-signed TLS certificates |

**Transport types:**

| Type | Use Case | Notes |
|------|----------|-------|
| `udp` | Native clients (best performance) | Direct UDP, lowest latency |
| `websocket` | Web clients (browser) | Required for WASM — runs over TCP |
| `webtransport` | Web clients (experimental) | UDP-like for browsers, requires TLS |

For WebTransport, the `certificate_sans` list determines which hostnames the self-signed certificate is valid for. Add your server's hostname/IP if deploying remotely.

---

## Diagnostics

```json
{
  "diagnostics_log_interval_secs": 10.0
}
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `diagnostics_log_interval_secs` | f64 | `10.0` | Seconds between server diagnostics log entries |
