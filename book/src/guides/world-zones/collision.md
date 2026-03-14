# Collision Guide

How collision meshes work in this project — what's supported, how to set them up, and best practices.

---

## How It Works

Collision meshes live in the **zones GLB file** alongside spawn points, death zones, and other zone types. Collision nodes use the `collision_` prefix. The zone processor extracts meshes from the glTF scene hierarchy, preserves Blender transforms, and creates Avian3d physics colliders.

**Key files:**

- `crates/game-core/src/zones/processor.rs` — processes `collision_` prefixed nodes into Avian3d colliders
- `crates/game-core/src/world/mod.rs` — `WorldCollisionBundle` and mesh extraction helpers
- `crates/game-core/src/world/collision_debug.rs` — debug visualization (press `C`)

**Asset path:** Configured via `zones_path` in `game_core_config.json` (default: `models/world_zones.glb`)

---

## What's Supported

| Feature                  | Supported | Notes                                                            |
| ------------------------ | --------- | ---------------------------------------------------------------- |
| Trimesh colliders        | Yes       | Default for static world geometry                                |
| Convex hull colliders    | Yes       | Via `create_convex_hull_collider()` for dynamic objects          |
| Compound colliders       | Yes       | Via `create_compound_collider()` for manual shape composition    |
| Static rigid bodies      | Yes       | All collision meshes are `RigidBody::Static`                     |
| Transform preservation   | Yes       | Node transforms from Blender are preserved                       |
| Named nodes              | Yes       | Each collision entity gets `Name::new("Collision: {node_name}")` |
| Debug visualization      | Yes       | Toggle with `C` key when collision debug is enabled              |
| Multiple mesh primitives | Yes       | All primitives per node are processed                            |
| `collision_` prefix      | Yes       | Required — nodes must be named `collision_floor`, etc.           |

---

## Collider Types

### Trimesh (Default)

- Used for: Static world geometry (floors, walls, terrain)
- Created automatically from `collision_` prefixed nodes in the zones GLB
- Handles complex concave shapes
- Best for: Static environment

### Convex Hull

- Used for: Dynamic objects that need physics
- Call `create_convex_hull_collider(&mesh)` manually
- Better performance than trimesh for moving objects
- Limitation: Approximates concave shapes as convex

### Compound

- Used for: Complex dynamic objects built from simple primitives
- Call `create_compound_collider(shapes)` with a list of (position, rotation, collider) tuples
- Best performance for objects that can be described with basic shapes

```rust
let collider = create_compound_collider(vec![
    (Vec3::ZERO, Quat::IDENTITY, Collider::cuboid(10.0, 1.0, 10.0)),       // floor
    (Vec3::new(5.0, 2.0, 0.0), Quat::IDENTITY, Collider::cuboid(1.0, 2.0, 10.0)), // wall
]);
```

---

## Blender Setup

Collision meshes now live in the **same file** as zones. In your Blender scene:

1. Name collision objects with the `collision_` prefix (e.g., `collision_floor`, `collision_walls`)
2. Keep them in the same collection as your zones, or a sub-collection
3. Simplify geometry aggressively — collision meshes should be 10-20x fewer polygons than visual

### Naming Convention

```
collision_floor      → Static floor collider
collision_walls      → Static wall collider
collision_ramp_01    → Static ramp collider
collision_platform   → Static platform collider
```

### Export Settings

Export alongside zones:

```
File → Export → glTF 2.0

Include:
  ✅ Selected Objects
  ✅ Custom Properties
  ❌ Cameras
  ❌ Punctual Lights

Transform:
  ✅ +Y Up

Geometry:
  ✅ Apply Modifiers
  ✅ Normals
  ❌ Tangents
  ❌ Vertex Colors

Materials:
  ❌ No Export

Save as: world_zones.glb
```

---

## Best Practices

### Creating Collision Meshes in Blender

1. **Simplify aggressively** — collision meshes should be 10-20x fewer polygons than visual
2. **Use basic shapes** where possible — boxes for walls, planes for floors
3. **Close all geometry** — no holes or gaps, or characters fall through
4. **Check normals** — face outward (use Face Orientation overlay in Blender)
5. **Match origins** — collision and visual meshes should share the same origin point

### Polygon Budget

| Geometry       | Visual Mesh         | Collision Mesh  |
| -------------- | ------------------- | --------------- |
| Flat floor     | 2-4 polys           | 1-2 polys       |
| Simple wall    | 100+ polys (detail) | 4-6 polys (box) |
| Curved surface | 1000+ polys         | 50-100 polys    |
| Full world     | 10,000+ polys       | 500-2,000 polys |

### Alignment

Both collision and visual meshes must align in world space:

- Apply all transforms before export (`Ctrl+A → All Transforms`)
- Export both at the same origin point
- Both are loaded at `Transform::default()` (origin) in Bevy

### Server vs Client

| Environment  | Visual | Collision   | Config                       |
| ------------ | ------ | ----------- | ---------------------------- |
| Server       | No     | Yes         | `ZonePluginConfig::server()` |
| Client       | Yes    | Yes         | `ZonePluginConfig::client()` |
| World Viewer | Yes    | Yes + debug | `ZonePluginConfig::viewer()` |

### Debug Visualization

When running with `ZonePluginConfig::viewer()` or `collision_debug: true`:

- Press `C` to toggle collision mesh visibility
- Collision meshes render as semi-transparent overlays
- Useful for verifying alignment between visual and collision meshes

---

## Troubleshooting

| Problem                           | Solution                                                      |
| --------------------------------- | ------------------------------------------------------------- |
| Character falls through floor     | Check collision mesh has no holes, normals face outward       |
| Collision offset from visual      | Apply transforms in both collections, export at same origin   |
| Collision not loading             | Check node names have `collision_` prefix                     |
| No collision debug visible        | Press `C`, ensure `collision_debug: true` in ZonePluginConfig |
| Collision too detailed            | Simplify mesh — high-poly collision hurts physics performance |
| "Failed to create collider" error | Mesh may have degenerate triangles or no valid geometry       |
