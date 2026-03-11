//! Client-side prediction helpers.
//!
//! Systems here maintain the speed scalar used to widen the position rollback
//! threshold at high speeds, and emit diagnostic logs during rollback
//! re-simulation.

use std::collections::HashMap;

use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;
use game_core::networking::protocol::{CharacterAction, CharacterMarker};
use game_core::networking::rollback::set_prediction_speed;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::*;

// ---------------------------------------------------------------------------
// Speed scalar
// ---------------------------------------------------------------------------

/// Updates the global speed used by `position_should_rollback` to scale the
/// threshold.
///
/// **Must run only on normal ticks** (`run_if(not(is_in_rollback))`).
/// If it ran during rollback re-simulation it would overwrite the current
/// speed with a historical value, temporarily narrowing or widening the
/// threshold and triggering more rollbacks.
pub fn update_prediction_speed(
    query: Query<&LinearVelocity, (With<Predicted>, With<CharacterMarker>)>,
) {
    let max_speed = query
        .iter()
        .map(|v| Vec2::new(v.x, v.z).length())
        .fold(0.0f32, f32::max);
    set_prediction_speed(max_speed);
}

// ---------------------------------------------------------------------------
// Rollback diagnostics
// ---------------------------------------------------------------------------

/// Logs each FixedUpdate tick that runs inside Lightyear rollback re-simulation.
pub fn log_rollback_tick(
    timeline: Res<LocalTimeline>,
    query: Query<(Entity, &LinearVelocity), With<Predicted>>,
) {
    let tick = timeline.tick();
    for (entity, vel) in &query {
        let speed = Vec2::new(vel.x, vel.z).length();
        warn!("[ROLLBACK-TICK] tick={tick:?} {entity:?} speed={speed:.3}");
    }
}

// ---------------------------------------------------------------------------
// Velocity-change detector
// ---------------------------------------------------------------------------

/// Detects velocity spikes each tick.
///
/// A sudden speed *increase* at the start of a tick (before any forces are
/// applied) is the signature of a rollback restoring a higher-speed
/// historical state.  The `[VEL-JUMP]` log is the primary signal that an
/// infinite-rollback loop is happening.
///
/// Normal per-tick changes are logged at `trace!` to keep the output quiet.
pub fn track_velocity_changes(
    mut last_speeds: Local<HashMap<Entity, f32>>,
    query: Query<(Entity, &LinearVelocity, &ActionState<CharacterAction>), With<Predicted>>,
) {
    for (entity, vel, action_state) in &query {
        let horiz_speed = Vec2::new(vel.x, vel.z).length();
        let input = action_state
            .axis_pair(&CharacterAction::Move)
            .clamp_length_max(1.0);
        let look_yaw = action_state.axis_pair(&CharacterAction::Look).x;
        let is_braking = input.length_squared() < 0.001;

        if let Some(&prev_speed) = last_speeds.get(&entity) {
            // Only log when something interesting is happening (avoids spam at rest)
            if prev_speed > 0.05 || horiz_speed > 0.05 {
                if horiz_speed > prev_speed + 0.1 {
                    warn!(
                        "[VEL-JUMP] {entity:?} speed {prev_speed:.3}→{horiz_speed:.3} \
                         vel=({:.3},{:.3})  input=({:.3},{:.3})  look_yaw={:.3}  braking={is_braking} \
                         — possible rollback restore",
                        vel.x, vel.z, input.x, input.y, look_yaw,
                    );
                } else {
                    trace!(
                        "[vel] {entity:?} {prev_speed:.3}→{horiz_speed:.3}  \
                         input=({:.3},{:.3})  look_yaw={:.3}  braking={is_braking}",
                        input.x, input.y, look_yaw,
                    );
                }
            }
        }

        last_speeds.insert(entity, horiz_speed);
    }
}
