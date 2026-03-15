# Configuration Overview

All game settings are driven by JSON files in `assets/config/`. Each file maps to a Rust struct with `#[serde(default)]`, so you can include only the fields you want to override — missing fields use defaults.

---

## Config Files

| File | Purpose | Used By |
|------|---------|---------|
| [game_client_config.json](client.md) | Window, input, rendering, transport, camera, debug | Native client, web client |
| [game_server_config.json](server.md) | Bind address, spawning, transport, diagnostics | Server |
| [game_simulation_config.json](simulation.md) | Movement physics, character, projectiles, zones | Server + client (shared) |
| [game_performance_config.json](performance.md) | Tick rate, network timing, rollback thresholds, vsync | Server + client (shared) |
| [game_world_config.json](world.md) | Asset paths, skybox, logging | Server + client (shared) |
| [dynamic_objects_config.json](dynamic-objects.md) | Dynamic object triggers, actions, state, light effects | Server + client (shared) |

---

## How Config Loading Works

Config loading uses `load_config<T>(filename)` in `crates/game-core/src/utils/config_loader.rs`:

1. Checks `ASSET_PATH` env var for the base directory (default: `assets`)
2. Reads `{base}/config/{filename}`
3. Deserializes JSON into the target struct
4. Falls back to `T::default()` if the file is missing or unparseable

All structs use `#[serde(default)]` — you can create a partial JSON file with only the fields you want to change.

---

## Environment Variable Overrides

Some settings can also be set via environment variables (see `.env.example`):

| Variable | Overrides | Default |
|----------|-----------|---------|
| `SERVER_HOST` | Server bind/connect address | `127.0.0.1` |
| `SERVER_PORT` | Server port | `5888` |
| `FIXED_TIMESTEP_HZ` | Tick rate | `64` |
| `INTERPOLATION_BUFFER_MS` | Client interpolation buffer | `100` |
| `RUST_LOG` | Log filter | Config file value |
| `CERT_PATH` | TLS certificate path | None |
| `KEY_PATH` | TLS key path | None |
| `ASSET_PATH` | Base asset directory | `assets` |

Environment variables take precedence over config file values when the application explicitly checks for them.

---

## Hot Reload

Config files are hot-reloaded at runtime — edit a JSON file while the game is running and changes apply automatically. The system polls for file modification every 1 second. On parse error, the old config is kept and a warning is logged.

**Not available on WASM** (web client) — the filesystem watcher is compiled out.

**What's watched per app:**

| Config File | Server | Client | World Viewer |
|-------------|--------|--------|--------------|
| `game_simulation_config.json` | Yes | Yes | Yes |
| `game_performance_config.json` | Yes | Yes | Yes |
| `game_world_config.json` | Yes | Yes | Yes |
| `game_server_config.json` | Yes | — | — |
| `game_client_config.json` | — | Yes | — |
| `game_camera_config.json` | — | — | Yes |
| `dynamic_objects_config.json` | Yes | Yes | Yes |

This is especially useful with the **world viewer** (`cargo dev-viewer`) — tweak camera sensitivity, movement physics, or world asset paths and see changes instantly without restarting.

**Key file:** `crates/game-core/src/utils/config_hot_reload.rs`

---

## Tips

- **Partial files are fine** — only include fields you want to change from defaults
- **Hot-reload is supported** — edit config JSON while running and changes apply within 1 second (native only, not WASM)
- **Validation** — invalid JSON logs a warning and falls back to defaults (or keeps old config during hot-reload)
- **Paths** — asset paths in configs are relative to the `assets/` directory
