# Baked Lighting Guide

How to bake diffuse and indirect lighting in Blender and bring it into Bevy — via vertex colors or lightmap textures.

---

## How It Works

Baked lighting captures the full lighting solution (direct + indirect + bounced light) and stores it in the mesh data. This gives the scene realistic global illumination without any runtime light computation.

There are two approaches:

1. **Vertex color baking** — lighting stored per-vertex, simplest workflow, no extra textures
2. **Lightmap texture baking** — lighting stored in a texture on UV1, higher quality, larger file size

Both are Blender-side workflows. Bevy reads vertex colors and textures from the GLB automatically — no custom Rust code needed.

**When to use baked lighting:**
- Static scenes with fixed light positions
- Indoor environments with complex indirect/bounced light
- Performance-critical scenes (mobile, web client)
- When you want global illumination without runtime cost

**When to use realtime lighting instead:**
- Dynamic time of day or moving lights
- Small number of simple lights where runtime cost is acceptable
- See [Lighting Guide](lighting.md) for realtime light setup

---

## What's Supported

| Feature | Supported | Notes |
|---|---|---|
| Vertex color light bake | Yes | Per-vertex, simple workflow |
| Lightmap texture bake | Yes | Via UV1 channel, higher quality |
| Direct lighting | Yes | Light hitting surfaces directly |
| Indirect / bounced lighting | Yes | Light bouncing off nearby surfaces (global illumination) |
| Color bleeding | Yes | Colored surfaces tinting nearby geometry |
| Emissive light contribution | Yes | Emissive materials act as light sources during bake |
| Dynamic objects | No | Baked lighting is static |

---

## Approach 1: Vertex Color Baking

Best for: quick iteration, low-poly scenes, when texture memory is limited.

### 1. Set Up Lights in Blender

Place your lights as they should appear in the final scene:

- **Sun light** for outdoor directional light
- **Point/spot lights** for indoor light sources
- **Emissive materials** on surfaces that should emit light (lamps, screens, lava)

### 2. Create a Vertex Color Layer

1. Select the mesh
2. **Object Data Properties** → **Color Attributes** → click **+**
3. Name it `BakedLight`
4. Set **Domain** to `Face Corner`, **Data Type** to `Byte Color`

### 3. Bake the Lighting

1. Switch to **Cycles** render engine
2. Go to **Render Properties** → **Bake**
3. Set **Bake Type** to `Diffuse`
4. Under **Contributions**, enable:
   - **Direct** — light hitting surfaces
   - **Indirect** — bounced/global illumination
   - **Color** — disable this if you want lighting only (no albedo baked in)
5. Set **Target** to `Active Color Attribute`
6. Select your meshes and click **Bake**

#### Bake Settings

| Setting | Recommended | Notes |
|---|---|---|
| Samples | 256-1024 | Higher = cleaner, less noise |
| Bounces | 3-5 | Number of light bounces for indirect lighting |
| Color contribution | Off | Bake lighting only, keep albedo in textures |
| Margin | 2px | Not critical for vertex colors |

### 4. Connect to Material

Multiply the baked light into the albedo:

1. **Shader Editor** → add **Color Attribute** node (set to `BakedLight`)
2. Add a **Mix** node set to **Multiply**
3. Connect: Image Texture (albedo) → Mix input 1
4. Connect: Color Attribute (baked light) → Mix input 2
5. Connect: Mix output → Principled BSDF Base Color

### 5. Export

Same as [Meshes Guide](meshes.md) with **Vertex Colors** enabled in Geometry settings.

---

## Approach 2: Lightmap Texture Baking

Best for: high-quality results, detailed lighting on low-poly meshes, large surfaces.

### 1. Create a Lightmap UV Channel

1. Select the mesh
2. **Object Data Properties** → **UV Maps** → click **+** to add a second UV map
3. Name it `Lightmap` (this becomes UV1)
4. With the mesh selected, go to Edit Mode → select all faces
5. **UV → Lightmap Pack** (or Smart UV Project with Island Margin = 0.02)
   - Ensure no overlapping UVs — every face needs unique UV space

### 2. Create a Lightmap Texture

1. Open the **Shader Editor**
2. Add an **Image Texture** node (don't connect it to anything yet)
3. Click **New** to create a new image:
   - Name: `Lightmap`
   - Resolution: 1024x1024 or 2048x2048
   - Color: black
4. **Select this node** (it must be active/selected for baking)

### 3. Bake to Texture

1. Switch to **Cycles** render engine
2. **Render Properties** → **Bake**
3. Set **Bake Type** to `Diffuse`
4. Enable **Direct** + **Indirect**, disable **Color**
5. Set **Target** to `Image Textures`
6. Make sure the lightmap UV map is the **active UV** for rendering (click the camera icon next to it)
7. Select meshes → click **Bake**
8. Save the baked image: **Image Editor** → **Image → Save As**

### 4. Connect the Lightmap

1. In the **Shader Editor**, set the Image Texture node's UV input to the `Lightmap` UV map
2. Use a **Mix** node (Multiply) to combine:
   - Albedo texture (using UV0) → Mix input 1
   - Lightmap texture (using UV1) → Mix input 2
   - Mix output → Principled BSDF Base Color

### 5. Export

The lightmap texture is embedded in the GLB. Make sure to:

- **Pack the lightmap image** before export: `File → External Data → Pack Resources`
- Both UV channels (UV0 + UV1) export automatically
- Enable **Images (embedded)** in export Materials settings

---

## Combining with Baked Shadows

For the best results, combine baked lighting with [baked shadows (AO)](baked-shadows.md):

1. Bake AO to vertex colors (see [Baked Shadows Guide](baked-shadows.md))
2. Bake lighting to vertex colors or lightmap texture (this guide)
3. Multiply both into the final material:
   - Albedo × Baked Light × Baked AO → Base Color

This gives you full global illumination plus contact shadows in crevices, all with zero runtime cost.

---

## Best Practices

- **Disable Color contribution** when baking Diffuse — bake lighting only, keep albedo in textures. This lets you change textures without re-baking.
- **Use high sample counts** (512+) for clean indirect lighting — noise is very visible in baked results.
- **Vertex colors are lower quality** but simpler. Use lightmap textures when you need smooth gradients on large surfaces.
- **Lightmap UV must not overlap** — every face needs unique space. Use Lightmap Pack or Smart UV Project.
- **Test in world-viewer** — `cargo dev-viewer` shows the final result without networking overhead.
- **Bake at final geometry** — apply modifiers before baking.
- **Consider file size** — lightmap textures add to the GLB size. Use 1024x1024 unless you need more detail.

---

## Troubleshooting

| Problem | Solution |
|---|---|
| Baked light not visible | Check Vertex Colors enabled in export, or lightmap texture is packed and embedded |
| Light looks flat/uniform | Increase bake Samples, check that lights are positioned correctly in Blender |
| Noise/grain in baked result | Increase Samples (512+), add Denoising in Render Properties |
| Lightmap UV overlapping | Re-unwrap UV1 with Lightmap Pack, increase Island Margin |
| Black areas in lightmap | Check for missing UVs (faces not unwrapped) or flipped normals |
| Color bleeding too strong | Reduce light bounce count, or adjust material colors |
| Bake button grayed out | Switch to Cycles (baking requires Cycles, not Eevee) |
| Lightmap texture not in GLB | Pack resources before export, check Images is set to Embedded |
