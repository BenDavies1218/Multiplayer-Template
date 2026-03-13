//! Server-side authoritative character movement.
//!
//! The server reads inputs from the replicated `ActionState<CharacterAction>`,
//! extracts the camera yaw from the Look axis, then delegates to
//! `apply_character_movement` — the same shared function used by the client.

use avian3d::prelude::*;
use bevy::prelude::*;
use game_core::GameCoreConfig;
use game_networking::movement::apply_character_movement;
use game_networking::protocol::{CameraOrientation, CharacterAction, CrouchState};
use leafwing_input_manager::prelude::*;

/// Apply camera-relative movement to every character (authoritative).
///
/// Runs each `FixedUpdate` tick on the server.
#[allow(clippy::type_complexity)]
pub fn handle_character_actions(
    spatial_query: SpatialQuery,
    mut query: Query<(
        Entity,
        &ActionState<CharacterAction>,
        &Position,
        &mut LinearVelocity,
        &mut CrouchState,
        &mut CameraOrientation,
    )>,
    config: Res<GameCoreConfig>,
) {
    for (entity, action_state, position, mut linear_velocity, mut crouch_state, mut cam_orient) in
        &mut query
    {
        let look = action_state.axis_pair(&CharacterAction::Look);
        let camera_yaw = look.x;

        // Update CameraOrientation from the replicated ActionState —
        // needed for projectile spawn direction.
        cam_orient.yaw = look.x;
        cam_orient.pitch = look.y;

        apply_character_movement(
            entity,
            &mut linear_velocity,
            &spatial_query,
            action_state,
            position,
            camera_yaw,
            &mut crouch_state,
            &config.movement,
            &config.character,
            config.networking.fixed_timestep_hz as f32,
        );
    }
}
