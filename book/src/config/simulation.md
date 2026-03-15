# Simulation Config

**File:** `assets/config/game_simulation_config.json`
**Struct:** `GameSimulationConfig` in `crates/game-core/src/simulation_config.rs`

Shared by server and client. Controls movement physics, character dimensions, projectile behavior, and zone parameters. Both sides must use the same values for prediction to work correctly.

---

## Movement

Physics parameters for character movement.

```json
{
  "movement": {
    "max_speed": 5.0,
    "max_acceleration": 20.0,
    "max_deceleration": 40.0,
    "jump_impulse": 5.0,
    "sprint_multiplier": 1.8,
    "crouch_multiplier": 0.4,
    "crouch_capsule_height": 0.25,
    "ground_tolerance": 0.15
  }
}
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `max_speed` | f32 | `5.0` | Maximum movement speed (m/s) |
| `max_acceleration` | f32 | `20.0` | How fast the character reaches max speed (m/s²) |
| `max_deceleration` | f32 | `40.0` | How fast the character stops when no input (m/s²) |
| `jump_impulse` | f32 | `5.0` | Upward force applied on jump |
| `sprint_multiplier` | f32 | `1.8` | Speed multiplier while sprinting (effective speed = max_speed × sprint_multiplier) |
| `crouch_multiplier` | f32 | `0.4` | Speed multiplier while crouching |
| `crouch_capsule_height` | f32 | `0.25` | Capsule height when crouching (smaller = lower crouch) |
| `ground_tolerance` | f32 | `0.15` | Distance below feet to detect ground contact (meters) |

---

## Character

Character collision and model definitions.

```json
{
  "character": {
    "capsule_radius": 0.5,
    "capsule_height": 0.5,
    "model_catalog": {
      "default": "models/characters/default/player.glb"
    },
    "hitbox_regions": {
      "head": {
        "damage": 2.0,
        "shape": { "type": "Capsule", "radius": 0.15, "half_height": 0.1 }
      },
      "chest": {
        "damage": 1.0,
        "shape": { "type": "Box", "half_extents": [0.25, 0.3, 0.15] }
      },
      "arm": {
        "damage": 0.75,
        "shape": { "type": "Capsule", "radius": 0.08, "half_height": 0.2 }
      },
      "leg": {
        "damage": 0.5,
        "shape": { "type": "Capsule", "radius": 0.1, "half_height": 0.25 }
      }
    }
  }
}
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `capsule_radius` | f32 | `0.5` | Character collision capsule radius |
| `capsule_height` | f32 | `0.5` | Character collision capsule height (half-height of the cylindrical section) |
| `model_catalog` | map | 1 default entry | Maps model IDs to GLB asset paths |
| `hitbox_regions` | map | 4 regions | Named hitbox regions with damage multipliers and shapes |

**Hitbox shapes:**

| Shape | Parameters | Example |
|-------|-----------|---------|
| `Capsule` | `radius`, `half_height` | `{"type": "Capsule", "radius": 0.15, "half_height": 0.1}` |
| `Box` | `half_extents` [x, y, z] | `{"type": "Box", "half_extents": [0.25, 0.3, 0.15]}` |

The `damage` value is a multiplier — `2.0` means double damage (headshots), `0.5` means half damage (legs).

---

## Projectile

```json
{
  "projectile": {
    "velocity": 50.0,
    "lifetime_ms": 5000,
    "radius": 0.1
  }
}
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `velocity` | f32 | `50.0` | Projectile travel speed (m/s) |
| `lifetime_ms` | u64 | `5000` | How long a projectile exists before despawning (milliseconds) |
| `radius` | f32 | `0.1` | Projectile collision radius |

---

## Zones

Default parameters for zone-based effects (damage zones, death zones, spawn points).

```json
{
  "zones": {
    "default_damage": 10.0,
    "default_damage_interval": 1.0,
    "default_spawn_position": [0.0, 3.0, 0.0]
  }
}
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `default_damage` | f32 | `10.0` | Damage dealt per tick in damage zones |
| `default_damage_interval` | f32 | `1.0` | Seconds between damage ticks |
| `default_spawn_position` | [f32; 3] | `[0.0, 3.0, 0.0]` | Fallback spawn position if no spawn points defined in the zones GLB |
