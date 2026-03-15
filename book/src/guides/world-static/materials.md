# Materials Guide

How PBR materials, textures, and UV mapping work — from Blender's Principled BSDF to Bevy's `StandardMaterial`.

---

## How It Works

Blender's **Principled BSDF** shader maps directly to glTF's PBR material model, which Bevy loads as `StandardMaterial`. When you export a `.glb`, material properties and textures are embedded and applied automatically — no code needed.

**Key pipeline:** Principled BSDF (Blender) → glTF PBR (export) → `StandardMaterial` (Bevy)

---

## What's Supported

| Feature | Supported | Notes |
|---|---|---|
| Principled BSDF | Yes | Maps 1:1 to glTF PBR |
| Base color / albedo | Yes | Color value or texture |
| Base color texture | Yes | Image texture connected to Base Color input |
| Normal maps | Yes | Export with Tangents enabled in Geometry settings |
| Metallic / roughness | Yes | Combined or separate maps |
| Emissive color | Yes | Set emission color/strength — combine with bloom for glow |
| Emissive texture | Yes | Image texture connected to Emission input |
| Vertex colors | Yes | Enable Vertex Colors in export Geometry settings |
| Alpha / transparency | Yes | Set Alpha value or connect alpha texture |
| Occlusion texture | Yes | Connect to glTF Settings → Occlusion in Principled BSDF |
| Multiple UV channels | Yes | UV0 for textures, UV1 for lightmaps |
| Mapping node | No | Tiling/scale must be baked into UVs — see below |
| Generated/Object coordinates | No | Use UV output from Texture Coordinate only |
| Blender-specific nodes | No | Mix Shader, Fresnel, Layer Weight, etc. won't export |
| Shader-to-RGB | No | Blender-only — not part of glTF |

---

## PBR Material Properties

These Principled BSDF inputs export to glTF:

| Principled BSDF Input | glTF Property | Bevy `StandardMaterial` Field |
|---|---|---|
| Base Color | `baseColorFactor` / `baseColorTexture` | `base_color` / `base_color_texture` |
| Metallic | `metallicFactor` | `metallic` |
| Roughness | `roughnessFactor` | `perceptual_roughness` |
| Normal (via Normal Map node) | `normalTexture` | `normal_map_texture` |
| Emission Color | `emissiveFactor` / `emissiveTexture` | `emissive` / `emissive_texture` |
| Alpha | `alphaMode` / `alphaCutoff` | `alpha_mode` |
| IOR | `ior` (KHR_materials_ior) | `ior` |

### Setting Up a Basic Material

1. Select your mesh in Blender
2. Open the Shader Editor
3. Add a **Principled BSDF** node (usually added by default)
4. Connect your textures:
   - Image Texture → **Base Color** input for albedo
   - Image Texture → **Normal Map** node → **Normal** input for normals
   - Image Texture → **Metallic** / **Roughness** inputs for PBR maps
5. Set values directly for uniform properties (e.g., Metallic = 1.0 for metal)

### Emissive Materials

Emissive materials glow without needing a light source. Useful for screens, lava, neon signs, indicator lights.

1. In Principled BSDF, set **Emission Color** to the desired color
2. Set **Emission Strength** > 0 (higher = brighter glow)
3. Optionally connect an Image Texture to **Emission** for patterned glow
4. In Bevy, enable bloom on the camera for the glow halo effect

---

## UV Mapping

Texture tiling and scale must be baked into the UV coordinates. **Mapping nodes in Blender's shader graph do not export to glTF** — they are silently stripped.

### The Problem

If you use a Mapping node (Texture Coordinate → Mapping → Image Texture) to control texture tiling/scale in Blender, the texture will look correct in Blender but appear at the wrong scale in Bevy.

### The Fix

1. Note your Mapping node's Scale values (e.g., X=5, Y=5)
2. Select the mesh → Edit Mode → `A` to select all faces
3. Open UV Editor → `A` to select all UVs
4. `S` then type the scale value (e.g., `5`) → Enter
5. Remove the Mapping node — connect Texture Coordinate UV output directly to Image Texture Vector input
6. Re-export the GLB

The UVs now bake in the tiling that the Mapping node was doing, and the texture will look the same in both Blender and Bevy.

### What Exports

| Shader Feature | Exports | Notes |
|---|---|---|
| UV coordinates | Yes | Primary way to control texture scale/tiling |
| Mapping node (Location/Rotation/Scale) | No | Bake into UVs instead |
| Multiple UV channels | Yes | UV0 for textures, UV1 for lightmaps |
| Texture Coordinate node (UV output) | Yes | Connect directly to Image Texture |
| Texture Coordinate node (Generated/Object) | No | Use UV output only |

### Multiple UV Channels

You can use two UV channels per mesh:

- **UV0** — primary texture mapping (albedo, normal, roughness, etc.)
- **UV1** — secondary channel, typically used for lightmap textures or baked lighting

To add a second UV channel in Blender: select mesh → Object Data Properties → UV Maps → add a second entry.

---

## Textures

### Recommended Settings

| Setting | Recommendation |
|---|---|
| Resolution | 2048x2048 for world geometry, 1024x1024 for small props |
| Format | PNG or JPEG (embedded in GLB) |
| Color space | sRGB for color textures, Non-Color for normal/roughness/metallic |
| Packing | Always pack before export: `File → External Data → Pack Resources` |

### Texture Packing

For PBR workflows, glTF uses a combined **ORM texture** (Occlusion + Roughness + Metallic in R/G/B channels). Blender's exporter handles this automatically when you connect separate textures to the Principled BSDF inputs.

### Transparency

For transparent or alpha-cutoff materials:

1. Set the **Alpha** value on Principled BSDF (or connect an alpha texture)
2. In Blender's Material Settings → Surface → Blend Mode:
   - **Alpha Clip** — hard cutoff (foliage, fences)
   - **Alpha Blend** — smooth transparency (glass, water)
3. The exporter maps this to glTF `alphaMode` (`MASK` or `BLEND`)

---

## Best Practices

- Use **Principled BSDF** exclusively — it's the only shader that maps to glTF PBR
- Avoid Blender-specific shader nodes (Mix Shader, Fresnel, Layer Weight, etc.)
- Pack textures before export: `File → External Data → Pack Resources`
- Use reasonable texture sizes — 2048x2048 is usually sufficient for world geometry
- Set normal map textures to **Non-Color** color space in Blender
- For tiling textures, bake the tiling into UVs (don't rely on Mapping nodes)
- Test materials in the world viewer first: `cargo dev-viewer`

---

## Troubleshooting

| Problem | Solution |
|---|---|
| Model appears gray/untextured | Check Materials Export is enabled, verify Principled BSDF is connected |
| Textures missing | Pack resources before export (`File → External Data → Pack Resources`) |
| Texture scale wrong | Mapping nodes don't export — bake tiling into UVs (see UV Mapping above) |
| Normal map looks flat | Enable Tangents in Geometry export settings |
| Material too shiny/dull | Adjust Roughness value (0 = mirror, 1 = matte) |
| Transparency not working | Set Blend Mode in Blender material settings, check Alpha value |
| Emissive not glowing | Increase Emission Strength, enable bloom on camera in Bevy |
| Colors look wrong | Check color space: sRGB for color textures, Non-Color for data textures |
