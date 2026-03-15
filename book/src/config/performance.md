# Performance Config

**File:** `assets/config/game_performance_config.json`
**Struct:** `GamePerformanceConfig` in `crates/game-core/src/performance_config.rs`

Shared by server and client. Controls tick rate, network timing, rollback thresholds, and rendering performance. These values directly affect the feel and responsiveness of the game.

---

## Networking

```json
{
  "networking": {
    "fixed_timestep_hz": 64.0,
    "send_interval_hz": 64.0,
    "interpolation_buffer_ms": 100,
    "client_timeout_secs": 3,
    "protocol_id": 0
  }
}
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `fixed_timestep_hz` | f64 | `64.0` | Server simulation tick rate (Hz). Higher = smoother but more CPU |
| `send_interval_hz` | f64 | `64.0` | Network packet send rate (Hz). Usually matches tick rate |
| `interpolation_buffer_ms` | u64 | `100` | Client interpolation buffer (ms). Higher = smoother but more latency |
| `client_timeout_secs` | i32 | `3` | Seconds before server considers a client disconnected |
| `protocol_id` | u64 | `0` | Lightyear protocol identifier. Must match between server and client |

**Tuning tips:**

- **Tick rate 64 Hz** is standard for FPS games. Lower (32 Hz) saves CPU, higher (128 Hz) improves hit registration.
- **Interpolation buffer** should be at least 2× the send interval. At 64 Hz send rate (15.6ms per packet), 100ms gives ~6 packets of buffer.
- **Client timeout** should be short enough to detect disconnects quickly but long enough to survive brief network hiccups.

---

## Rollback Thresholds

Controls when client predictions are reconciled with authoritative server state. When the difference between predicted and server state exceeds these thresholds, a rollback occurs.

```json
{
  "rollback_thresholds": {
    "position": 5.0,
    "position_speed_factor": 0.3,
    "rotation": 0.1,
    "linear_velocity": 0.5,
    "angular_velocity": 1.0
  }
}
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `position` | f32 | `5.0` | Base position error threshold (meters) |
| `position_speed_factor` | f32 | `0.3` | Additional threshold per unit of player speed |
| `rotation` | f32 | `0.1` | Rotation error threshold (radians) |
| `linear_velocity` | f32 | `0.5` | Velocity difference threshold (m/s) |
| `angular_velocity` | f32 | `1.0` | Angular velocity difference threshold (rad/s) |

**How position threshold works:**

The effective position threshold is dynamic:

```
effective_threshold = position + (player_speed × position_speed_factor)
```

At rest (speed 0): threshold = 5.0m
At sprint speed (~9 m/s): threshold = 5.0 + (9.0 × 0.3) = 7.7m

This prevents unnecessary rollbacks during fast movement where prediction naturally drifts more.

**Tuning tips:**

- **Lower thresholds** = more rollbacks, better accuracy, more visible corrections
- **Higher thresholds** = fewer rollbacks, smoother feel, less accurate
- **Position 5.0** is very permissive — tighten to 1.0-2.0 for competitive gameplay
- **Rotation 0.1** radians ≈ 5.7 degrees — reasonable for most cases

---

## Display

```json
{
  "enable_diagnostics": true,
  "vsync": true
}
```

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `enable_diagnostics` | bool | `true` | Enable performance diagnostics logging |
| `vsync` | bool | `true` | Enable vertical sync (caps framerate to monitor refresh rate) |
