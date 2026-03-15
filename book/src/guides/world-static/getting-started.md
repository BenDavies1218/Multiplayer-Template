# World Static — Getting Started

A quick overview of how static world assets work in this project and how to get your first Blender scene into Bevy.

---

## Overview

Static world assets are the unchanging geometry, materials, lighting, and effects that make up the environment. They're authored in **Blender**, exported as **glTF/GLB** files, and loaded by Bevy at startup.

The pipeline looks like this:

```
Blender scene → Export as .glb → assets/models/ → Bevy SceneRoot → rendered in-game
```

Everything in the GLB is preserved automatically — mesh hierarchy, transforms, PBR materials, textures, lights, vertex colors, and animations.

---

## What Goes Into a Static World

| Layer | What It Is | Guide |
|---|---|---|
| Meshes | Geometry, scene hierarchy, transforms | [Meshes](meshes.md) |
| Materials | PBR shading, textures, UV mapping | [Materials](materials.md) |
| Baked Shadows | Ambient occlusion / contact shadows via vertex colors | [Baked Shadows](baked-shadows.md) |
| Baked Lighting | Global illumination baked to vertex colors or lightmaps | [Baked Lighting](baked-lighting.md) |
| Lighting | Realtime punctual lights (point, spot, directional) | [Lighting](lighting.md) |
| Skybox | HDR environment cubemap | [Skybox](skybox.md) |
| Baked Animations | Looping ambient animations (fans, floating objects) | [Baked Animations](baked-animations.md) |

---

## Quick Start Workflow

### 1. Set Up Blender

- Use **Blender 3.6+** (glTF 2.0 exporter included)
- Set units to **Metric** (1 Blender unit = 1 meter in Bevy)
- Use **Principled BSDF** shader for all materials (it maps 1:1 to glTF PBR)

### 2. Build Your Scene

- Model geometry as static meshes
- Apply PBR materials with textures (albedo, normal, roughness, metallic)
- Add lights (point, spot, sun) for realtime lighting
- Optionally bake AO/lighting to vertex colors for extra detail

### 3. Export to GLB

```
File → Export → glTF 2.0

Include:       ✅ Selected Objects, ✅ Punctual Lights
Transform:     ✅ +Y Up
Geometry:      ✅ Apply Modifiers, ✅ UVs, ✅ Normals, ✅ Tangents, ✅ Vertex Colors
Materials:     ✅ Export, ✅ Images (embedded)
Animation:     ✅ Animations, ✅ Always Sample Animations

Save as: your_world_visual.glb
```

**Before exporting:**
1. Select all objects to export
2. Apply all transforms: `Ctrl+A` → All Transforms
3. Pack textures: `File → External Data → Pack Resources`

### 4. Configure the Asset Path

Set the visual mesh path in `assets/config/game_core_config.json`:

```json
{
  "world_assets": {
    "visual_path": "models/your_world_visual.glb"
  }
}
```

### 5. Test in the World Viewer

```bash
cargo dev-viewer
```

This launches the standalone viewer without networking — fastest way to check how your scene looks in Bevy.

---

## Recommended Reading Order

If you're new to the world asset pipeline, read the guides in this order:

1. **[Meshes](meshes.md)** — geometry fundamentals and export settings
2. **[Materials](materials.md)** — PBR shading, textures, UV mapping
3. **[Lighting](lighting.md)** — realtime lights from Blender
4. **[Baked Shadows](baked-shadows.md)** — ambient occlusion for depth
5. **[Baked Lighting](baked-lighting.md)** — global illumination without runtime cost
6. **[Skybox](skybox.md)** — environment backgrounds
7. **[Baked Animations](baked-animations.md)** — ambient motion
