# World Zones Guide

How to define gameplay zones and collision in Blender (spawn points, death zones, damage zones, triggers, collision geometry) and export them for use in the game.

---

## How It Works

All world data (collision geometry, spawn points, death zones, damage zones, triggers) lives in a single GLB file. The zone processor parses node names by prefix to determine their type and creates the appropriate entities.

**Key files:**

- `crates/game-core/src/zones/` — zone plugin, loader, processor, collision detection

**Asset path:** Configured via `zones_path` in `game_core_config.json` (default: `models/world_zones.glb`)

---

## Node Types

| Type        | Node Prefix    | Collider              | Behavior                                         |
| ----------- | -------------- | --------------------- | ------------------------------------------------ |
| Collision   | `collision_`   | Trimesh (static)      | Static world physics geometry                    |
| Spawn Point | `spawn_`       | None (transform only) | Defines where players spawn/respawn              |
| Death Zone  | `deathzone_`   | Sensor trimesh        | Instant kill — teleports player to a spawn point |
| Damage Zone | `damage_`      | Sensor trimesh        | Logs damage (future: health system integration)  |
| Trigger     | `trigger_`     | Sensor trimesh        | Fires `ZoneEnteredEvent` / `ZoneExitedEvent`     |

---

## Blender Setup

### 1. Create a Zones Collection

Create a dedicated collection in Blender (e.g., `World_zones`). This keeps all world data (collision + zones) together.

### 2. Add Objects

#### Collision Geometry

Add simplified meshes that define the physical boundaries of the world:

```
collision_floor      → Static floor collider
collision_walls      → Static wall collider
collision_ramp_01    → Static ramp collider
```

See the [Collision Guide](COLLISION_GUIDE.md) for best practices on collision mesh creation.

#### Zone Objects

For each zone, add a simple mesh that defines the zone's area:

- **Spawn points**: Place an empty or small cube where players should spawn. Only the position matters.
- **Death zones**: Create a mesh covering the kill area (e.g., a flat plane under the map for void death).
- **Damage zones**: Same as death zones but for areas that deal damage over time.
- **Triggers**: Any volume that should fire an event when a player enters/exits.

### 3. Name Your Objects

The node name prefix determines the type:

```
collision_floor    → Static physics collider
collision_walls    → Static physics collider
spawn_01           → Spawn point, index 1
spawn_02           → Spawn point, index 2
deathzone_void     → Death zone named "void"
deathzone_lava     → Death zone named "lava"
damage_acid        → Damage zone named "acid"
damage_fire        → Damage zone named "fire"
trigger_door       → Generic trigger named "door"
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

Save as: world_zones.glb
```

---

## Server vs Client

| Environment  | Collision | Zones | Detection | Debug |
| ------------ | --------- | ----- | --------- | ----- |
| Server       | Yes       | Yes   | Yes       | No    |
| Client       | Yes       | Yes   | No        | No    |
| World Viewer | Yes       | Yes   | No        | Yes   |

The server is fully authoritative over zone logic. The client loads collision for physics prediction but does not run zone detection.

---

## Debug Visualization

The world viewer includes debug visualization for both collision and zones.

**Toggle keys:**
- Press `C` to show/hide collision debug meshes
- Press `Z` to show/hide zone debug meshes

**Color coding:**

| Type        | Color                        |
| ----------- | ---------------------------- |
| Collision   | Red (30% opacity)            |
| Death Zone  | Red (30% opacity)            |
| Damage Zone | Yellow (30% opacity)         |
| Trigger     | Blue (30% opacity)           |
| Spawn Point | Green sphere (30% opacity)   |

Debug meshes are semi-transparent overlays so you can verify placement matches the visual world. Debug visualization is only available in the world viewer (`ZonePluginConfig::viewer()`).

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
- Use the same origin point as your visual meshes
- Zone positions should match the visual world (e.g., death zone under the lava mesh)

---

## Troubleshooting

| Problem                             | Solution                                                                              |
| ----------------------------------- | ------------------------------------------------------------------------------------- |
| Players don't spawn at spawn points | Check server logs for "Registered N spawn points"                                     |
| Death zone doesn't trigger          | Ensure mesh has volume (not a single plane), check node name starts with `deathzone_` |
| Custom properties not parsed        | Enable "Custom Properties" in export settings                                         |
| Zone positions are wrong            | Apply transforms in Blender before export                                             |
| No zones loading                    | Check `zones_path` in `game_core_config.json`                                        |
| Debug meshes not showing            | Press `Z`/`C` to toggle, only works in world viewer                                  |
| Unrecognized node prefix warning    | Node names must start with: `collision_`, `spawn_`, `deathzone_`, `damage_`, `trigger_` |
