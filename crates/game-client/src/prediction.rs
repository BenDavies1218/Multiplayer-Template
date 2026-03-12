//! Client-side prediction helpers.
//!
//! Maintains the speed scalar used to widen the position rollback
//! threshold at high speeds.

use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;
use game_networking::protocol::CharacterMarker;
use game_networking::rollback::set_prediction_speed;
use lightyear::prelude::*;

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
