# Skybox Guide

How to set up and pre-bake skybox images for the game client.

---

## How It Works

The client loads a skybox image (EXR format) and applies it as a cubemap texture on the camera. Bevy requires cubemap textures (6 faces) for the `Skybox` component — equirectangular images must be converted first.

**Key file:** `crates/game-client/src/renderer.rs` (skybox loading and conversion)
**Key file:** `assets/config/game_core_config.json` (skybox path config)
**Key file:** `tools/bake_skybox_cubemap/bake_skybox_cubemap.py` (offline cubemap baking)

---

## Supported Formats

| Format                    | Supported | Notes                                                   |
| ------------------------- | --------- | ------------------------------------------------------- |
| Stacked cubemap EXR (1:6) | Yes       | Best option — no runtime conversion needed              |
| Equirectangular EXR (2:1) | Yes       | Converted to cubemap at runtime (slow for large images) |
| Other aspect ratios       | Partial   | Attempts stacked reinterpret if dimensions align        |
| PNG/JPEG                  | No        | Use EXR for HDR skyboxes                                |

---

## Configuration

Set the skybox path in `assets/config/game_core_config.json`:

```json
{
  "world_assets": {
    "skybox_path": "sunset_sky_hdr_cubemap.exr"
  }
}
```

The path is relative to the `assets/` directory.

---

## Pre-Baking a Cubemap

Runtime conversion of large equirectangular images blocks the main thread and can cause network timeouts. Pre-baking converts the image offline so loading is instant.

### Requirements

```bash
pip install numpy openexr
```

### Usage

From the project root:

```bash
# Default: 1024 face size, reads assets/sunset_sky_hdr.exr
python3 tools/bake_skybox_cubemap/bake_skybox_cubemap.py

# Custom face size (higher = better quality, larger file)
python3 tools/bake_skybox_cubemap/bake_skybox_cubemap.py --face-size 2048

# Custom input/output paths
python3 tools/bake_skybox_cubemap/bake_skybox_cubemap.py -i assets/my_skybox.exr -o assets/my_skybox_cubemap.exr
```

### Options

| Flag                | Default                     | Description                |
| ------------------- | --------------------------- | -------------------------- |
| `-i`, `--input`     | `assets/sunset_sky_hdr.exr` | Input equirectangular EXR  |
| `-o`, `--output`    | `<input>_cubemap.exr`       | Output stacked cubemap EXR |
| `-s`, `--face-size` | `1024`                      | Cube face size in pixels   |

### After Baking

Update `skybox_path` in `assets/config/game_core_config.json` to point to the baked file:

```json
{
  "world_assets": {
    "skybox_path": "sunset_sky_hdr_cubemap.exr"
  }
}
```

### Face Size Recommendations

| Face Size | Output Dimensions | File Size (approx) | Use Case                         |
| --------- | ----------------- | ------------------ | -------------------------------- |
| 512       | 512 x 3072        | ~24 MB             | Fast iteration, low-end hardware |
| 1024      | 1024 x 6144       | ~96 MB             | Good balance of quality and size |
| 2048      | 2048 x 12288      | ~384 MB            | High quality, large file         |

---

## Runtime Conversion (Fallback)

If you use an equirectangular image without pre-baking, the renderer converts it to a cubemap at startup. This works but has downsides:

- **Blocks the main thread** — a 16384x8192 EXR takes ~8 seconds
- **Can cause network timeouts** — the client can't process packets during conversion
- **Runs every launch** — wasted CPU time on every startup

The renderer caps runtime face size at 2048 to limit conversion time. For best results, always pre-bake.

---

## Getting Skybox Images

Equirectangular HDR/EXR skyboxes are available from:

- [Poly Haven](https://polyhaven.com/hdris) — free CC0 HDRIs
- [ambientCG](https://ambientcg.com/) — free CC0 HDRIs
- Blender — render a 360 panorama from your scene

Download as EXR format, then run the bake script before using in-game.

---

## Troubleshooting

| Problem                                   | Cause                                                  | Fix                                                            |
| ----------------------------------------- | ------------------------------------------------------ | -------------------------------------------------------------- |
| `Texture binding expects Cube but got D2` | Skybox image not converted to cubemap                  | Use a pre-baked cubemap or ensure the renderer conversion runs |
| Game freezes on startup                   | Large equirectangular image being converted at runtime | Pre-bake the cubemap offline                                   |
| Network timeout on connect                | Skybox conversion blocking the main thread             | Pre-bake the cubemap offline                                   |
| Black skybox                              | Image failed to load or has zero dimensions            | Check the file path and format                                 |
| Skybox looks pixelated                    | Face size too small                                    | Re-bake with a larger `--face-size`                            |
