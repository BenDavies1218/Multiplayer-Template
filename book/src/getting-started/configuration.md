# Configuration

All game settings are driven by JSON config files in `assets/config/`.

## Config Files

| File | Purpose |
|------|---------|
| `game_core_config.json` | Asset path, networking (host, port, tick rate), movement, physics, world assets, zones, dynamic objects |
| `game_client_config.json` | Window settings, input bindings, rendering, transport |
| `game_camera_config.json` | Camera modes (first-person, third-person, free-view), sensitivity |
| `game_server_config.json` | Projectile settings, spawning, transport type, certificate SANs |

## How Config Loading Works

Config loading is done via `load_config<T>(filename)` in `crates/game-core/src/utils/config_loader.rs`.

- All config structs use `#[serde(default)]` so partial JSON files work
- If a config file is missing entirely, `T::default()` is used
- The `ASSET_PATH` environment variable overrides the default `assets` directory

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ASSET_PATH` | `../../assets` | Base path for configs and assets |
| `RUST_LOG` | `info` | Log level |
| `SERVER_HOST` | `127.0.0.1` | Server bind address |
| `SERVER_PORT` | `5888` | Server port |

## Example: Network Settings

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
