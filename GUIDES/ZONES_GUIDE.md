# World Zones Guide

How to define gameplay zones in Blender (spawn points, death zones, damage zones, triggers) and export them for use in the game.

---

## How It Works

Zones are loaded from a separate GLB file (`example_world_zones.glb`). The server loads this file, parses node names and custom properties, and creates sensor colliders for gameplay logic. The client never loads zones — it only sees the effects (e.g., player teleporting after entering a death zone).

**Key files:**

- `crates/game-core/src/zones/` — zone plugin, loader, processor, collision detection
- `crates/game-core/src/shared.rs` — `WORLD_ZONES_PATH` constant

**Asset path:** `assets/models/example_world_zones.glb`

---

## Zone Types

| Type        | Node Prefix  | Collider              | Behavior                                         |
| ----------- | ------------ | --------------------- | ------------------------------------------------ |
| Spawn Point | `spawn_`     | None (transform only) | Defines where players spawn/respawn              |
| Death Zone  | `deathzone_` | Sensor trimesh        | Instant kill — teleports player to a spawn point |
| Damage Zone | `damage_`    | Sensor trimesh        | Logs damage (future: health system integration)  |
| Trigger     | `trigger_`   | Sensor trimesh        | Fires `ZoneEnteredEvent` / `ZoneExitedEvent`     |

---

## Blender Setup

### 1. Create a Zones Collection

Create a dedicated collection in Blender (e.g., `ExampleWorld_zones`). This keeps zones separate from visual and collision meshes.

### 2. Add Zone Objects

For each zone, add a simple mesh (cube, plane, or custom shape) that defines the zone's area:

- **Spawn points**: Place an empty or small cube where players should spawn. Only the position matters.
- **Death zones**: Create a mesh covering the kill area (e.g., a flat plane under the map for void death, a box around lava).
- **Damage zones**: Same as death zones but for areas that deal damage over time.
- **Triggers**: Any volume that should fire an event when a player enters/exits.

### 3. Name Your Objects

The node name prefix determines the zone type:

```
spawn_01          → Spawn point, index 1
spawn_02          → Spawn point, index 2
deathzone_void    → Death zone named "void"
deathzone_lava    → Death zone named "lava"
damage_acid       → Damage zone named "acid"
damage_fire       → Damage zone named "fire"
trigger_door      → Generic trigger named "door"
trigger_checkpoint → Generic trigger named "checkpoint"
```

The numeric suffix on spawn points determines spawn order (round-robin when players connect).

### 4. Add Custom Properties (Optional)

Select an object, then in the Properties panel add custom properties:

**For Damage Zones:**

| Property   | Type  | Default | Description                  |
| ---------- | ----- | ------- | ---------------------------- |
| `damage`   | Float | 10.0    | Damage per tick              |
| `interval` | Float | 1.0     | Seconds between damage ticks |

**For Trigger Zones:**

| Property    | Type   | Default   | Description                                  |
| ----------- | ------ | --------- | -------------------------------------------- |
| `event`     | String | node name | Custom event name                            |
| `animation` | String | —         | Animation to play (future feature)           |
| `target`    | String | —         | Target entity for animation (future feature) |

Custom properties are exported as glTF extras JSON and parsed by the zone processor.

---

## Export Settings

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

Save as: example_world_zones.glb
```

---

## Server vs Client

| Environment  | Loads Zones | Detection | Debug |
| ------------ | ----------- | --------- | ----- |
| Server       | Yes         | Yes       | No    |
| Client       | No          | No        | No    |
| World Viewer | Yes         | No        | Yes   |

The server is fully authoritative over zone logic. The client never loads the zones file.

---

## Debug Visualization

The world viewer includes debug visualization for zones, similar to collision debug (C key).

**Toggle:** Press `Z` to show/hide zone debug meshes.

**Color coding:**

| Zone Type   | Color                        |
| ----------- | ---------------------------- |
| Death Zone  | Red (30% opacity)            |
| Damage Zone | Yellow (30% opacity)         |
| Trigger     | Blue (30% opacity)           |
| Spawn Point | Green sphere (30% opacity)   |

Debug meshes are semi-transparent overlays rendered on top of zone colliders so you can verify placement matches the visual world. Spawn points display as small spheres at their position.

Debug visualization is only available in the world viewer (`ZonePluginConfig::viewer()`). The server and client never render debug meshes.

**Key files:**

- `crates/game-core/src/zones/zone_debug.rs` — `ZoneDebugSettings`, toggle and visibility systems
- `crates/game-core/src/zones/processor.rs` — spawns debug meshes alongside zone entities

---

## Best Practices

### Zone Meshes

- Use simple geometry — cubes and planes are ideal
- Zone meshes don't render in-game, only their collision shape matters
- Make zones slightly larger than needed to ensure reliable detection
- Don't overlap death zones with spawn points

### Spawn Points

- Place at least 2 spawn points for multiplayer
- Position spawn points above ground level (Y + 1-2 units) so players don't spawn inside the floor
- Number them sequentially: `spawn_01`, `spawn_02`, etc.
- If no spawn points are defined, players spawn at (0, 3, 0) as fallback

### Alignment

- Apply all transforms before export: select all → `Ctrl+A` → All Transforms
- Use the same origin point as your visual and collision meshes
- Zone positions should match the visual world (e.g., death zone under the lava mesh)

---

## Troubleshooting

| Problem                             | Solution                                                                              |
| ----------------------------------- | ------------------------------------------------------------------------------------- |
| Players don't spawn at spawn points | Check server logs for "Registered N spawn points"                                     |
| Death zone doesn't trigger          | Ensure mesh has volume (not a single plane), check node name starts with `deathzone_` |
| Custom properties not parsed        | Enable "Custom Properties" in export settings                                         |
| Zone positions are wrong            | Apply transforms in Blender before export                                             |
| No zones loading                    | Check file is at `assets/models/example_world_zones.glb`                              |
| Debug meshes not showing            | Press `Z` to toggle, only works in world viewer (not server/client)                   |
