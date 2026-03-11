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

use crate::core_config::RollbackConfig;

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
pub fn position_should_rollback(this: &Position, that: &Position) -> bool {
    let diff = this.0 - that.0;
    let horiz_dist = Vec2::new(diff.x, diff.z).length();
    let cfg = rollback_thresholds();
    let speed = prediction_speed();
    let threshold = cfg.position + speed * cfg.position_speed_factor;

    if horiz_dist >= threshold {
        warn!(
            "[rollback:position] horiz={horiz_dist:.4}m >= threshold={threshold:.4}m \
             (base={:.3} + speed={speed:.2}×factor={:.3})  \
             pred=({:.3},{:.3},{:.3})  server=({:.3},{:.3},{:.3})",
            cfg.position,
            cfg.position_speed_factor,
            this.x, this.y, this.z,
            that.x, that.y, that.z,
        );
    }
    horiz_dist >= threshold
}

/// Returns `true` when rotational divergence exceeds the threshold (radians).
pub fn rotation_should_rollback(this: &Rotation, that: &Rotation) -> bool {
    let angle = this.angle_between(*that);
    let threshold = rollback_thresholds().rotation;
    if angle >= threshold {
        debug!("[rollback:rotation] angle={angle:.4}rad >= threshold={threshold:.4}rad");
    }
    angle >= threshold
}

/// Returns `true` when linear velocity diverges by more than the threshold (m/s).
pub fn linear_velocity_should_rollback(this: &LinearVelocity, that: &LinearVelocity) -> bool {
    let diff = (this.0 - that.0).length();
    let threshold = rollback_thresholds().linear_velocity;
    if diff >= threshold {
        warn!(
            "[rollback:linear_velocity] diff={diff:.4}m/s >= threshold={threshold:.4}m/s  \
             pred=({:.3},{:.3},{:.3})  server=({:.3},{:.3},{:.3})",
            this.x, this.y, this.z,
            that.x, that.y, that.z,
        );
    }
    diff >= threshold
}

/// Returns `true` when angular velocity diverges by more than the threshold (rad/s).
pub fn angular_velocity_should_rollback(this: &AngularVelocity, that: &AngularVelocity) -> bool {
    let diff = (this.0 - that.0).length();
    let threshold = rollback_thresholds().angular_velocity;
    if diff >= threshold {
        debug!(
            "[rollback:angular_velocity] diff={diff:.4}rad/s >= threshold={threshold:.4}rad/s"
        );
    }
    diff >= threshold
}
