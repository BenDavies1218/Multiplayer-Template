# World Config

**File:** `assets/config/game_world_config.json`
**Struct:** `GameWorldConfig` in `crates/game-core/src/world_config.rs`

Shared by server and client. Controls asset paths for world meshes and the skybox, plus logging configuration.

---

## World Assets

Paths to the GLB files that make up the world. All paths are relative to the `assets/` directory.

```json
{
  "world_assets": {
    "visual_path": "models/world_static.glb",
    "dynamic_path": "models/world_dynamic.glb",
    "zones_path": "models/world_zones.glb",
    "skybox_path": "sunset_sky_hdr_cubemap.exr"
  }
}
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `visual_path` | string | `"models/world_static.glb"` | Visual mesh — what players see (client only) |
| `dynamic_path` | string | `"models/world_dynamic.glb"` | Dynamic objects mesh (triggers, interactive objects) |
| `zones_path` | string | `"models/world_zones.glb"` | Collision and zone mesh (collision_, spawn_, deathzone_, damage_, trigger_ prefixes) |
| `skybox_path` | string | `"sunset_sky_hdr_cubemap.exr"` | Skybox image — stacked cubemap EXR recommended |

**How each file is used:**

| File | Loaded By | Server | Client |
|------|-----------|--------|--------|
| Visual mesh | `WorldPlugin` | Not loaded | Full scene with materials, lights, animations |
| Dynamic mesh | `DynamicPlugin` | Loaded for trigger/state logic | Loaded for visuals + trigger effects |
| Zones mesh | `ZonePlugin` | Loaded for collision + zone detection | Loaded for collision (no visuals) |
| Skybox | `ClientSkyboxPlugin` | Not loaded | Applied to camera as cubemap |

---

## Asset Path

Override the base directory for all assets.

```json
{
  "asset_path": "assets"
}
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `asset_path` | string | `"assets"` | Base directory for all asset loading |

Can also be set via the `ASSET_PATH` environment variable.

---

## Logging

```json
{
  "logging": {
    "default_level": "info",
    "filter": "wgpu=error,bevy_render=info,bevy_ecs=warn,bevy_time=warn,naga=warn"
  }
}
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `default_level` | string | `"info"` | Default log level: `"debug"`, `"info"`, `"warn"`, `"error"` |
| `filter` | string | *(see below)* | Tracing filter string — per-module log levels |

**Filter syntax:** Comma-separated `module=level` pairs. Example:

```
wgpu=error,bevy_render=info,bevy_ecs=warn,game_networking=debug
```

The `RUST_LOG` environment variable overrides this when set.
