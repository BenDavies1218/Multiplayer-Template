# World Viewer

A standalone world asset viewer for quick testing without networking dependencies.

## Purpose

This tool lets you test your world visual and collision meshes quickly without running the full client/server setup. Perfect for:
- Testing Blender exports
- Verifying world textures load correctly
- Checking collision mesh generation
- Iterating on world design

## Usage

```bash
# Quick start with fast dev build
cargo dev-viewer

# Or standard build
cargo run -p world-viewer
```

## Controls

- **WASD** - Move camera
- **Space** - Move camera up
- **Shift** - Move camera down
- **Ctrl** - Speed boost (3x speed)
- **Mouse** - Look around
- **Left Click** - Grab/lock cursor
- **Escape** - Release cursor

## What It Loads

The viewer automatically loads:
- Visual mesh: `assets/models/example_world_visual.glb`
- Collision mesh: `assets/models/example_world_collision.glb`
- Lighting preset: Outdoor day lighting
- Test capsule: Green capsule for scale reference

## Dependencies

Minimal dependencies compared to the full game:
- Bevy (rendering only)
- Avian3D (physics/collision)
- game-core (world loading system)

No networking, no lightyear, no multiplayer overhead!
