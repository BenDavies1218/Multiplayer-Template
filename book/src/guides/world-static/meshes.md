# Meshes Guide

How visual meshes work in this project — geometry, scene hierarchy, export settings, and best practices.

For materials, textures, and UV mapping, see the [Materials Guide](materials.md).

---

## How It Works

Visual meshes are loaded as a Bevy `SceneRoot` from a `.glb` file. The entire scene hierarchy from Blender is preserved — transforms, parent-child relationships, and materials all come through automatically.

**Key file:** `crates/game-core/src/world/loader.rs`
**Asset path:** `assets/models/example_world_visual.glb`

---

## What's Supported

| Feature              | Supported | Notes                                                                 |
| -------------------- | --------- | --------------------------------------------------------------------- |
| Static meshes        | Yes       | Any geometry exports fine                                             |
| Scene hierarchy      | Yes       | Parent-child relationships preserved from Blender                     |
| Transforms           | Yes       | Position, rotation, scale come through automatically                  |
| PBR materials        | Yes       | See [Materials Guide](materials.md)                                   |
| Embedded textures    | Yes       | Recommended — keeps everything in one `.glb`                          |
| External textures    | No        | Use embedded textures instead                                         |
| Skeletal meshes      | Not used  | Supported by Bevy but not set up in this project                      |
| Morph targets        | Not used  | Supported by Bevy but not set up in this project                      |
| Animations           | Yes       | Looping ambient animations auto-play — see [Baked Animations](baked-animations.md) |
| Punctual lights      | Yes       | See [Lighting Guide](lighting.md)                                     |

---

## Blender Export Settings

```
File → Export → glTF 2.0

Include:
  ✅ Selected Objects
  ✅ Punctual Lights    (if scene contains lights — see [Lighting Guide](lighting.md))

Transform:
  ✅ +Y Up

Geometry:
  ✅ Apply Modifiers
  ✅ UVs
  ✅ Normals
  ✅ Tangents
  ✅ Vertex Colors (if used)

Materials:
  ✅ Export
  ✅ Images (embedded)

Animation:
  ✅ Animations
  ✅ Always Sample Animations

Save as: example_world_visual.glb
```

### Export Checklist

Before exporting, verify:

1. All objects to export are **selected** (meshes + lights)
2. All transforms are **applied**: `Ctrl+A` → All Transforms
3. Textures are **packed**: `File → External Data → Pack Resources`
4. Materials use **Principled BSDF** only (Blender-specific nodes won't export)

---

## Best Practices

### Scene Organization

- Use a dedicated Blender collection for visual meshes (e.g., `ExampleWorld_visual`)
- Keep collision meshes in a separate collection — they're exported independently
- Apply all transforms before export: select all → `Ctrl+A` → All Transforms

### Performance

- Keep polygon counts reasonable — Bevy renders the full visual mesh on every client
- Use LOD (Level of Detail) manually if needed by creating simplified versions
- Combine meshes that share materials to reduce draw calls
- Target 5-50MB for the exported `.glb` file

### Coordinate System

- Blender: Z-up, Bevy: Y-up — the glTF exporter handles this with +Y Up / -Z Forward
- 1 Blender unit = 1 meter in Bevy
- Set your scene origin at (0, 0, 0) — this is where the mesh spawns in Bevy

---

## Troubleshooting

| Problem                         | Solution                                                          |
| ------------------------------- | ----------------------------------------------------------------- |
| Wrong orientation               | Apply transforms in Blender, use +Y Up / -Z Forward               |
| Wrong scale                     | Apply scale in Blender (`Ctrl+A` → Scale), 1 unit = 1 meter       |
| Huge file size                  | Reduce texture resolution, use Draco compression, decimate mesh    |
| `missing field nodes` error     | Export had no objects selected — select all meshes + lights first   |
| Lights not appearing            | Enable Punctual Lights in export Include tab (see [Lighting Guide](lighting.md)) |
| Model appears gray/untextured   | See [Materials Guide](materials.md) troubleshooting                |
| Texture scale wrong             | See [Materials Guide](materials.md) — UV Mapping section           |
