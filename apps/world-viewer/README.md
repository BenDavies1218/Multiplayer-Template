# World Viewer

A standalone world asset viewer for quick testing without networking dependencies.

## Purpose

This tool lets you test your world visual and collision meshes quickly without running the full client/server setup. Perfect for:

- Testing Blender exports
- Verifying world textures load correctly
- Checking collision mesh generation
- Iterating on world design
- Testing zone configurations (damage zones, spawn points, triggers)

## Usage

```bash
# Quick start with fast dev build (dynamic linking)
cargo dev-viewer

# Or standard build
cargo run -p world-viewer
```

## Controls

- **WASD** — Move camera
- **Space** — Move camera up
- **Shift** — Move camera down
- **Ctrl** — Speed boost (3x speed)
- **Mouse** — Look around
- **Left Click** — Grab/lock cursor
- **Escape** — Release cursor
- **C** — Toggle collision mesh visualization

## What It Loads

The viewer loads world assets configured in `assets/config/game_core_config.json`:

- **Visual mesh**: `assets/models/example_world_visual.glb`
- **Collision mesh**: `assets/models/example_world_collision.glb`
- **Zone mesh**: `assets/models/example_world_zones.glb`
- **Lighting**: Outdoor day lighting preset
- **Test capsule**: Physics-enabled capsule that falls and collides with the world (uses character dimensions from config)

The viewer window opens at **1920x1080** resolution.

## Dependencies

Minimal dependencies compared to the full game:

- Bevy (rendering only)
- Avian3D (physics/collision)
- game-core (world/zone loading and configuration)

No networking, no lightyear, no multiplayer overhead.

## Related Documentation

- [Root README](../../README.md)
- [Server README](../server/README.md)
- [Native Client README](../native/README.md)
- [Web Client README](../web/README.md)
