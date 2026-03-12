//! Rollback threshold configuration and comparison functions.
//!
//! The free functions here are registered with Lightyear via `.add_should_rollback(fn)`
//! in `ProtocolPlugin`. Lightyear calls them whenever it receives a confirmed server
//! snapshot and needs to decide whether to re-simulate.
//!
//! # Dynamic speed-scaled threshold
//!
//! `position_should_rollback` uses a speed-scaled threshold so that normal prediction
//! offset (the client is always a few ticks ahead of the server's last ack) does not
//! trigger spurious rollbacks.  The formula is:
//!
//!   threshold = base_position + current_speed × speed_factor
//!
//! `set_prediction_speed` must be called from a client FixedUpdate system that is
//! **guarded by `not(is_in_rollback)`**.  If it runs during rollback re-simulation it
//! will overwrite the speed with a historical value, destabilising the threshold.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

use avian3d::prelude::*;
use bevy::prelude::*;

use game_core::core_config::RollbackConfig;

// ---------------------------------------------------------------------------
// Global config — set once at startup
// ---------------------------------------------------------------------------

static ROLLBACK_CONFIG: OnceLock<RollbackConfig> = OnceLock::new();

/// Initialise the global rollback thresholds from a [`RollbackConfig`].
///
/// Must be called once during plugin setup, before any rollback checks run.
/// Safe to call multiple times — subsequent calls are silently ignored.
pub fn init_rollback_config(config: RollbackConfig) {
    if ROLLBACK_CONFIG.set(config).is_err() {
        warn!("[rollback] init_rollback_config called more than once — ignoring duplicate");
    }
}

fn rollback_thresholds() -> &'static RollbackConfig {
    ROLLBACK_CONFIG.get_or_init(RollbackConfig::default)
}

// ---------------------------------------------------------------------------
// Per-tick speed — updated by a client system, read by comparison functions
// ---------------------------------------------------------------------------

static CURRENT_SPEED: AtomicU32 = AtomicU32::new(0);

/// Store the current predicted character's horizontal speed (m/s).
///
/// Call this from client `FixedUpdate` **with `run_if(not(is_in_rollback))`**.
/// If called during rollback re-simulation the stored value will be a historical
/// speed, making the position threshold temporarily wrong.
pub fn set_prediction_speed(speed: f32) {
    CURRENT_SPEED.store(speed.to_bits(), Ordering::Relaxed);
}

fn prediction_speed() -> f32 {
    f32::from_bits(CURRENT_SPEED.load(Ordering::Relaxed))
}

// ---------------------------------------------------------------------------
// Rollback comparison functions
// ---------------------------------------------------------------------------

/// Returns `true` when the horizontal (XZ) position divergence exceeds the
/// speed-scaled threshold.
///
/// Y-axis is intentionally excluded — Avian ground-contact caches can produce
/// a small Y discrepancy every tick even when the simulation is healthy.
pub fn position_should_rollback(predicted: &Position, confirmed: &Position) -> bool {
    let dx = predicted.x - confirmed.x;
    let dz = predicted.z - confirmed.z;
    let diff_xz = (dx * dx + dz * dz).sqrt();

    let cfg = rollback_thresholds();
    let speed = prediction_speed();
    let threshold = cfg.position + speed * cfg.position_speed_factor;

    let should = diff_xz > threshold;
    if should {
        info!(
            "[rollback] POSITION xz={diff_xz:.3} > threshold={threshold:.3} (base={:.2} speed={speed:.1} factor={:.2}) pred=({:.2},{:.2},{:.2}) conf=({:.2},{:.2},{:.2})",
            cfg.position,
            cfg.position_speed_factor,
            predicted.x,
            predicted.y,
            predicted.z,
            confirmed.x,
            confirmed.y,
            confirmed.z,
        );
    }
    should
}

/// Rotation rollback — disabled. Character rotation is locked via `LockedAxes`,
/// so rotation never diverges meaningfully.
pub fn rotation_should_rollback(_this: &Rotation, _that: &Rotation) -> bool {
    false
}

/// Returns `true` when the horizontal (XZ) linear velocity diverges by more
/// than the threshold.
///
/// Y-axis is excluded — vertical velocity is driven by Avian's gravity
/// integration and ground-collision response, which naturally differ between
/// client and server when positions have drifted.  Only XZ is set
/// deterministically by `apply_character_movement`.
pub fn linear_velocity_should_rollback(
    _predicted: &LinearVelocity,
    _confirmed: &LinearVelocity,
) -> bool {
    // Disabled — velocity is instantaneous and naturally differs between predicted
    // and confirmed ticks due to camera yaw changes during the prediction window.
    // Position rollback alone is sufficient to correct accumulated drift.
    false
}

/// Angular velocity rollback — disabled. Character rotation is locked via `LockedAxes`,
/// so angular velocity stays at zero.
pub fn angular_velocity_should_rollback(_this: &AngularVelocity, _that: &AngularVelocity) -> bool {
    false
}
