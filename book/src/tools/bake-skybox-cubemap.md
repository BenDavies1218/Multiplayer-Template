# bake-skybox-cubemap

A Python script that converts equirectangular EXR skybox images into stacked cubemap EXR files for instant loading in Bevy.

## Why Pre-Bake?

Bevy requires cubemap textures (6 faces) for the `Skybox` component. If you provide an equirectangular image, the renderer converts it at startup — but this blocks the main thread for several seconds and can cause network timeouts. Pre-baking offline eliminates this cost entirely.

The output format is a 1:6 stacked cubemap (face_size × face_size×6) with faces in +X, -X, +Y, -Y, +Z, -Z order. The renderer detects this aspect ratio and uses the fast `reinterpret_stacked_2d_as_array(6)` path — no CPU conversion at runtime.

## Requirements

```bash
pip install numpy openexr
```

## Usage

From the project root:

```bash
# Default: 1024 face size, reads assets/sunset_sky_hdr.exr
python3 tools/bake_skybox_cubemap/bake_skybox_cubemap.py

# Custom face size (higher = better quality, larger file)
python3 tools/bake_skybox_cubemap/bake_skybox_cubemap.py --face-size 2048

# Custom input/output paths
python3 tools/bake_skybox_cubemap/bake_skybox_cubemap.py -i assets/my_skybox.exr -o assets/my_skybox_cubemap.exr
```

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `-i`, `--input` | `assets/sunset_sky_hdr.exr` | Input equirectangular EXR |
| `-o`, `--output` | `<input>_cubemap.exr` | Output stacked cubemap EXR |
| `-s`, `--face-size` | `1024` | Cube face size in pixels |

## Face Size Recommendations

| Face Size | Output Dimensions | File Size (approx) | Use Case |
|-----------|-------------------|---------------------|----------|
| 512 | 512 x 3072 | ~24 MB | Fast iteration, low-end hardware |
| 1024 | 1024 x 6144 | ~96 MB | Good balance of quality and size |
| 2048 | 2048 x 12288 | ~384 MB | High quality, large file |

## After Baking

Update `skybox_path` in `assets/config/game_core_config.json` to point to the baked file:

```json
{
  "world_assets": {
    "skybox_path": "sunset_sky_hdr_cubemap.exr"
  }
}
```

## How It Works

1. Loads the equirectangular EXR image (2:1 aspect ratio)
2. For each of the 6 cube faces, computes 3D ray directions from pixel coordinates
3. Maps each ray to equirectangular UV coordinates
4. Samples the source image with bilinear interpolation
5. Stacks all 6 faces vertically into the output image
6. Saves as EXR with PIZ compression

The conversion uses vectorized NumPy for performance — processing all pixels per face in a single pass.

## Getting Skybox Images

Equirectangular HDR/EXR skyboxes are available from:

- [Poly Haven](https://polyhaven.com/hdris) — free CC0 HDRIs
- [ambientCG](https://ambientcg.com/) — free CC0 HDRIs
- Blender — render a 360 panorama from your scene

Download as EXR format, then run the bake script before using in-game.

See the [Skybox Guide](../guides/world-static/skybox.md) for full details on skybox configuration and runtime behavior.
