# Meshes Guide

How visual meshes work in this project — what's supported, how to set them up in Blender, and best practices.

---

## How It Works

Visual meshes are loaded as a Bevy `SceneRoot` from a `.glb` file. The entire scene hierarchy from Blender is preserved — transforms, parent-child relationships, and materials all come through automatically.

**Key file:** `crates/game-core/src/world/loader.rs`
**Asset path:** `assets/models/example_world_visual.glb`

---

## What's Supported

| Feature              | Supported | Notes                                                      |
| -------------------- | --------- | ---------------------------------------------------------- |
| Static meshes        | Yes       | Any geometry exports fine                                  |
| PBR materials        | Yes       | Principled BSDF maps directly to Bevy's `StandardMaterial` |
| Base color textures  | Yes       | Albedo/diffuse maps                                        |
| Normal maps          | Yes       | Export with Tangents enabled                               |
| Metallic/roughness   | Yes       | Combined or separate maps                                  |
| Emissive materials   | Yes       | Works with Bevy's bloom if enabled                         |
| Vertex colors        | Yes       | Enable in export settings                                  |
| Multiple UV channels | Yes       | UV0 for textures, UV1 for lightmaps                        |
| Embedded textures    | Yes       | Recommended — keeps everything in one `.glb`               |
| External textures    | No        | Use embedded textures instead                              |
| Skeletal meshes      | Not used  | Supported by Bevy but not set up in this project           |
| Morph targets        | Not used  | Supported by Bevy but not set up in this project           |
| Animations           | Not used  | World is static                                            |

---

## Blender Export Settings for Visual Meshes

```
File → Export → glTF 2.0

Include:
  ✅ Selected Objects

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

Save as: example_world_visual.glb
```

---

## Best Practices

### Scene Organization

- Use a dedicated Blender collection for visual meshes (e.g., `ExampleWorld_visual`)
- Keep collision meshes in a separate collection — they're exported independently
- Apply all transforms before export: select all → `Ctrl+A` → All Transforms

### Materials

- Use **Principled BSDF** shader — it maps 1:1 to glTF PBR
- Avoid Blender-specific shader nodes (they won't export)
- Pack textures before export: `File → External Data → Pack Resources`
- Use reasonable texture sizes — 2048x2048 is usually sufficient for world geometry

### Performance

- Keep polygon counts reasonable — Bevy renders the full visual mesh on every client
- Use LOD (Level of Detail) manually if needed by creating simplified versions
- Combine meshes that share materials to reduce draw calls
- Target 5-50MB for the exported `.glb` file

### Coordinate System

- Blender: Z-up, Bevy: Y-up — the glTF exporter handles this with +Y Up / -Z Forward
- 1 Blender unit = 1 meter in Bevy
- Set your scene origin at (0, 0, 0) — this is where the mesh spawns in Bevy

### Troubleshooting

| Problem                       | Solution                                                        |
| ----------------------------- | --------------------------------------------------------------- |
| Model appears gray/untextured | Check Materials tab has Export enabled, verify UV mapping       |
| Textures missing              | Pack resources before export, use embedded images               |
| Wrong orientation             | Apply transforms in Blender, use +Y Up / -Z Forward             |
| Wrong scale                   | Apply scale in Blender (`Ctrl+A → Scale`), 1 unit = 1 meter     |
| Huge file size                | Reduce texture resolution, use Draco compression, decimate mesh |
