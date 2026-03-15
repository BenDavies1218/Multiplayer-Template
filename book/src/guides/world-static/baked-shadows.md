# Baked Shadows Guide

How to bake ambient occlusion and contact shadows in Blender and bring them into Bevy via vertex colors.

---

## How It Works

Baked shadows are pre-computed in Blender and stored as **vertex colors** in the mesh. When exported to `.glb`, Bevy reads vertex colors automatically and applies them to the material. This gives the scene realistic shadow detail without any runtime shadow computation.

This is a Blender-side workflow — there's no custom Rust code needed. Bevy's glTF loader handles vertex colors natively.

**When to use baked shadows:**
- Static geometry that doesn't move (walls, floors, pillars, terrain)
- Ambient occlusion (soft shadows in crevices and corners)
- Contact shadows (where objects meet surfaces)
- Performance-sensitive scenes where realtime shadows are too expensive

**When to use realtime shadows instead:**
- Dynamic objects that move or are spawned at runtime
- Directional light shadows (sun) that change with time of day
- See [Lighting Guide](lighting.md) for realtime shadow setup

---

## What's Supported

| Feature | Supported | Notes |
|---|---|---|
| Ambient occlusion (AO) bake | Yes | Bake to vertex colors, export in GLB |
| Shadow bake | Yes | Bake full shadow pass to vertex colors |
| Combined bake (AO + shadow) | Yes | Multiply passes together in vertex colors |
| Per-vertex resolution | Yes | Limited by vertex density — more vertices = more detail |
| Texture-based bake | Partial | Requires UV1 lightmap channel — see [Baked Lighting](baked-lighting.md) |
| Dynamic objects | No | Baked shadows are static — use realtime for moving objects |

---

## Blender Workflow

### 1. Prepare the Mesh

Baked shadows are stored per-vertex, so shadow detail depends on vertex density. Add geometry where you need shadow detail:

- Use a **Subdivision Surface** modifier on areas needing soft shadow gradients
- Add **loop cuts** along edges where shadow transitions happen
- Flat surfaces with few vertices will have blocky shadows — subdivide as needed

**Tip:** You don't need high density everywhere. Focus on corners, crevices, and where objects meet surfaces.

### 2. Create a Vertex Color Layer

1. Select the mesh
2. Go to **Object Data Properties** (green triangle icon) → **Color Attributes**
3. Click **+** to add a new color attribute
4. Name it (e.g., `BakedAO` or `Shadow`)
5. Set **Domain** to `Face Corner` and **Data Type** to `Byte Color`

### 3. Bake Ambient Occlusion

1. Switch to **Cycles** render engine (baking requires Cycles)
2. Go to **Render Properties** → **Bake**
3. Set **Bake Type** to `Ambient Occlusion`
4. Under **Output**, set **Target** to `Active Color Attribute`
5. Make sure your vertex color layer from step 2 is the active one
6. Select the mesh and click **Bake**

#### AO Bake Settings

| Setting | Recommended | Notes |
|---|---|---|
| Samples | 128-512 | Higher = cleaner but slower |
| Distance | 1.0-5.0 | Max ray distance — controls how far AO reaches |
| Only Local | Off | Include shadows from nearby objects |
| Margin | 2px | Not critical for vertex colors, matters for texture bakes |

### 4. Bake Shadow Pass (Optional)

For harder shadows from specific light sources:

1. Set **Bake Type** to `Shadow`
2. Position your lights (sun, point, spot) where you want shadows cast
3. Set **Target** to `Active Color Attribute`
4. Click **Bake**

### 5. Combine AO + Shadows (Optional)

To multiply AO and shadow passes together:

1. Bake AO to one vertex color layer (e.g., `BakedAO`)
2. Bake shadows to another layer (e.g., `BakedShadow`)
3. Use Blender's **Vertex Paint** mode or a Geometry Nodes setup to multiply them into a final layer
4. Set the combined layer as the active color attribute for export

### 6. Connect to Material

For the baked shadows to appear in the exported GLB:

1. Open the **Shader Editor**
2. Add a **Color Attribute** node → set it to your baked vertex color layer
3. Connect it to the **Base Color** input of Principled BSDF using a **Multiply** node:
   - Image Texture (albedo) → Mix/Multiply → Base Color
   - Color Attribute (baked AO) → Mix/Multiply → Base Color
4. This multiplies the shadow into the albedo, darkening corners and crevices

**Alternative approach:** Instead of multiplying in the shader graph, you can bake the shadow directly into the albedo texture. This avoids needing vertex colors but means the shadow is permanently in the texture.

### 7. Export Settings

Use the standard visual mesh export settings with vertex colors enabled:

```text
Geometry:
  ✅ Apply Modifiers
  ✅ UVs
  ✅ Normals
  ✅ Tangents
  ✅ Vertex Colors    ← required for baked shadows
```

All other settings same as [Meshes Guide](meshes.md).

---

## Best Practices

- **Vertex density matters** — baked shadow quality is limited by mesh resolution. Subdivide where you need smooth shadow gradients.
- **Start with AO only** — ambient occlusion alone adds significant depth. Only add shadow passes if you need hard directional shadows.
- **Combine with realtime lighting** — baked AO for ambient detail + one realtime directional light (sun) for dynamic shadows is a good balance.
- **Don't over-subdivide** — more vertices = larger GLB file and more GPU memory. Target detail where it's visible.
- **Test in world-viewer** — run `cargo dev-viewer` to check how baked shadows look in Bevy before testing in multiplayer.
- **Bake at final geometry** — apply modifiers before baking so the vertex positions match the exported mesh.

---

## Troubleshooting

| Problem | Solution |
|---|---|
| Shadows not visible in Bevy | Check Vertex Colors is enabled in export Geometry settings |
| Shadows look blocky/banded | Increase mesh vertex density (subdivide) in shadow areas |
| AO too dark/light | Adjust Distance parameter in bake settings, or adjust in vertex paint mode |
| Bake button grayed out | Switch to Cycles render engine (baking requires Cycles, not Eevee) |
| Wrong vertex color layer baked | Make sure the target layer is set as Active Color Attribute before baking |
| Shadows on wrong objects | Select only the meshes you want to bake, enable "Selected to Active" if baking from high-poly |
| No difference between AO and no AO | AO Distance may be too small — increase it so rays reach nearby geometry |
