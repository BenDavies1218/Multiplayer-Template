# Lighting Guide

How to bring lighting from Blender into the world — what's supported and best practices.

---

## How It Works

Bevy supports the `KHR_lights_punctual` glTF extension. When you export lights from Blender with **Punctual Lights** enabled, they're included in the `.glb` file and automatically spawned as part of the `SceneRoot` — no extra code needed.

**Key file:** `crates/game-core/src/world/loader.rs` (loads the scene)
**Key file:** `crates/game-client/src/renderer.rs` (client rendering setup)

---

## What's Supported

| Feature | Supported | Notes |
|---------|-----------|-------|
| Point lights | Yes | Via `KHR_lights_punctual` in GLB |
| Spot lights | Yes | Via `KHR_lights_punctual` in GLB |
| Directional lights | Yes | Via `KHR_lights_punctual` in GLB (sun light) |
| Light color | Yes | Exported from Blender |
| Light intensity | Yes | May need manual tweaking — Blender and Bevy use different units |
| Light range | Yes | Exported from Blender |
| Spot angle | Yes | Inner/outer cone angles |
| Shadows | Partial | Bevy enables shadows per-light; GLB lights may need shadow config added in code |
| Area lights | No | Not part of glTF spec — use multiple point lights instead |
| Ambient light | No | Set in Bevy code, not exported from Blender |
| Emissive materials | Yes | Not a light source, but glows — combine with bloom for effect |
| Baked lightmaps | Not currently | Was previously implemented, could be re-added |
| Vertex color baking | Yes | Bake lighting to vertex colors in Blender, export with vertex colors |

---

## Blender Export Settings for Lights

To include lights in your visual mesh export, update the Include tab:

```
Include:
  ✅ Selected Objects
  ✅ Punctual Lights    ← enable this (was previously unchecked)
```

All other settings remain the same as the visual mesh export.

---

## Light Types in Blender → Bevy

### Point Light
- Blender: Add → Light → Point
- Bevy: `PointLight` component
- Use for: Torches, lamps, small light sources

### Spot Light
- Blender: Add → Light → Spot
- Bevy: `SpotLight` component
- Use for: Flashlights, focused beams, stage lighting

### Sun (Directional) Light
- Blender: Add → Light → Sun
- Bevy: `DirectionalLight` component
- Use for: Outdoor sunlight, moonlight
- Position doesn't matter — only rotation affects direction

### Area Light
- Blender: Add → Light → Area
- Bevy: **Not supported** in glTF
- Workaround: Use multiple point lights or emissive planes

---

## Best Practices

### Intensity

Blender and Bevy use different intensity scales. You may need to adjust:

- Blender uses watts for point/spot lights
- glTF uses candelas (lumens per steradian)
- Bevy interprets these values directly

If lights are too bright or too dim, adjust intensity in Blender and re-export, or override in code after the scene loads.

### Shadows

Lights exported from GLB have shadows **disabled by default** in Bevy. To enable shadows on imported lights, add a system that configures them after spawn:

```rust
fn configure_imported_lights(
    mut point_lights: Query<&mut PointLight, Added<PointLight>>,
    mut directional_lights: Query<&mut DirectionalLight, Added<DirectionalLight>>,
) {
    for mut light in &mut point_lights {
        light.shadows_enabled = true;
    }
    for mut light in &mut directional_lights {
        light.shadows_enabled = true;
    }
}
```

### Performance

- Limit shadow-casting lights — each shadow-casting light adds a render pass
- Use point lights without shadows for ambient fill
- One directional light (sun) with shadows is usually sufficient for outdoor scenes
- Keep spot light count reasonable (< 10 shadow-casting)

### Ambient Light

Ambient light can't come from the GLB. Set it in code:

```rust
commands.insert_resource(AmbientLight {
    color: Color::WHITE,
    brightness: 200.0,
});
```

### Alternative: Baked Lighting

For static scenes, baking lighting to vertex colors or lightmap textures gives better performance:

1. **Vertex colors**: Bake in Blender (`Render → Bake → Diffuse`), export with Vertex Colors enabled
2. **Lightmap textures**: Bake to a second UV channel, apply as a separate texture (requires custom shader or lightmap plugin)
3. **Emissive materials**: Set emissive color/texture for surfaces that should glow

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| No lights appear | Check Punctual Lights is enabled in export settings |
| Lights too bright/dim | Adjust intensity in Blender, or modify after spawn in code |
| No shadows | Shadows are off by default — enable via system (see above) |
| Area lights missing | Not supported in glTF — use point lights instead |
| Light position wrong | Apply transforms in Blender before export (`Ctrl+A`) |
