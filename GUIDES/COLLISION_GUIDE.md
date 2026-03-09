# Collision Guide

How collision meshes work in this project — what's supported, how to set them up, and best practices.

---

## How It Works

Collision uses a **separate low-poly GLB file** loaded independently from the visual mesh. The collision processor extracts meshes from the glTF scene hierarchy, preserves Blender transforms, and creates Avian3d physics colliders.

**Key files:**

- `crates/game-core/src/world/loader.rs` — loads the collision GLB
- `crates/game-core/src/world/processor.rs` — converts meshes to Avian3d colliders
- `crates/game-core/src/world/utils.rs` — vertex/index extraction helpers
- `crates/game-core/src/world/collision_debug.rs` — debug visualization (press `C`)

**Asset path:** `assets/models/example_world_collision.glb`

---

## What's Supported

| Feature                          | Supported | Notes                                                            |
| -------------------------------- | --------- | ---------------------------------------------------------------- |
| Trimesh colliders                | Yes       | Default for static world geometry                                |
| Convex hull colliders            | Yes       | Via `create_convex_hull_collider()` for dynamic objects          |
| Compound colliders               | Yes       | Via `create_compound_collider()` for manual shape composition    |
| Static rigid bodies              | Yes       | All collision meshes are `RigidBody::Static`                     |
| Transform preservation           | Yes       | Node transforms from Blender are preserved                       |
| Named nodes                      | Yes       | Each collision entity gets `Name::new("Collision: {node_name}")` |
| Debug visualization              | Yes       | Toggle with `C` key when `enable_debug: true`                    |
| Multiple mesh primitives         | Yes       | All primitives per node are processed                            |
| Separate collision file          | Yes       | Recommended approach for multiplayer                             |
| Naming convention (`_collision`) | Partial   | Mentioned in docs but not actively used in current code          |

---

## Collider Types

### Trimesh (Default)

- Used for: Static world geometry (floors, walls, terrain)
- Created automatically from the collision GLB
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

## Blender Export Settings for Collision Meshes

```
File → Export → glTF 2.0

Include:
  ✅ Selected Objects
  ❌ Everything else

Transform:
  ✅ +Y Up

Geometry:
  ✅ Apply Modifiers
  ✅ Normals
  ❌ Tangents
  ❌ Vertex Colors

Materials:
  ❌ No Export

Save as: example_world_collision.glb
```

---

## Best Practices

### Creating Collision Meshes in Blender

1. **Use a separate collection** (e.g., `Example_World_collision`)
2. **Simplify aggressively** — collision meshes should be 10-20x fewer polygons than visual
3. **Use basic shapes** where possible — boxes for walls, planes for floors
4. **Close all geometry** — no holes or gaps, or characters fall through
5. **Check normals** — face outward (use Face Orientation overlay in Blender)
6. **Match origins** — collision and visual collections should share the same origin point

### Polygon Budget

| Geometry       | Visual Mesh         | Collision Mesh  |
| -------------- | ------------------- | --------------- |
| Flat floor     | 2-4 polys           | 1-2 polys       |
| Simple wall    | 100+ polys (detail) | 4-6 polys (box) |
| Curved surface | 1000+ polys         | 50-100 polys    |
| Full world     | 10,000+ polys       | 500-2,000 polys |

### Alignment

Both meshes must align in world space:

- Apply all transforms in both collections before export (`Ctrl+A → All Transforms`)
- Export both at the same origin point
- Both are loaded at `Transform::default()` (origin) in Bevy

### Server vs Client

| Environment  | Visual | Collision   | Config                        |
| ------------ | ------ | ----------- | ----------------------------- |
| Server       | No     | Yes         | `WorldPluginConfig::server()` |
| Client       | Yes    | Yes         | `WorldPluginConfig::client()` |
| World Viewer | Yes    | Yes + debug | `WorldPluginConfig::viewer()` |

### Debug Visualization

When running with `WorldPluginConfig::viewer()` or `enable_debug: true`:

- Press `C` to toggle collision mesh visibility
- Collision meshes render as semi-transparent overlays
- Useful for verifying alignment between visual and collision meshes

---

## Troubleshooting

| Problem                           | Solution                                                      |
| --------------------------------- | ------------------------------------------------------------- |
| Character falls through floor     | Check collision mesh has no holes, normals face outward       |
| Collision offset from visual      | Apply transforms in both collections, export at same origin   |
| Collision not loading             | Check `WorldPluginConfig` has `load_collision: true`          |
| No collision debug visible        | Press `C`, ensure `enable_debug: true` in config              |
| Collision too detailed            | Simplify mesh — high-poly collision hurts physics performance |
| "Failed to create collider" error | Mesh may have degenerate triangles or no valid geometry       |
| Collision file too large          | Remove materials from export, simplify geometry               |
