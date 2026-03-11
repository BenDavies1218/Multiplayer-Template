//! Server-side authoritative character movement.
//!
//! The server reads inputs from the replicated `ActionState<CharacterAction>`,
//! updates `CameraOrientation` from the Look axis, then delegates to
//! `apply_character_movement` — the same shared function used by the client.
//!
//! Unlike the client, the server runs on *all* character entities (no
//! `With<Predicted>` filter) because it owns the authoritative simulation.

use avian3d::prelude::*;
use bevy::prelude::*;
use game_core::GameCoreConfig;
use game_core::movement::apply_character_movement;
use game_core::networking::protocol::{CameraOrientation, CharacterAction, CrouchState};
use leafwing_input_manager::prelude::*;

/// Apply camera-relative movement forces to every character.
///
/// Runs each `FixedUpdate` tick on the server.  The `CameraOrientation`
/// component is written here so the server always tracks where each client is
/// looking (used for projectile direction and debugging).
pub fn handle_character_actions(
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut query: Query<(
        Entity,
        &ComputedMass,
        &ActionState<CharacterAction>,
        &mut CameraOrientation,
        Forces,
        &mut CrouchState,
    )>,
    config: Res<GameCoreConfig>,
) {
    for (entity, mass, action_state, mut camera_orientation, forces, mut crouch_state) in
        &mut query
    {
        let look = action_state.axis_pair(&CharacterAction::Look);
        // Log whenever the camera orientation changes (indicates input is arriving)
        if (look.x - camera_orientation.yaw).abs() > 0.001
            || (look.y - camera_orientation.pitch).abs() > 0.001
        {
            trace!(
                "[SRV-LOOK] {entity:?} yaw {:.3}→{:.3}  pitch {:.3}→{:.3}",
                camera_orientation.yaw, look.x,
                camera_orientation.pitch, look.y,
            );
        }
        camera_orientation.yaw = look.x;
        camera_orientation.pitch = look.y;

        apply_character_movement(
            entity,
            mass,
            &time,
            &spatial_query,
            action_state,
            forces,
            camera_orientation.yaw,
            &mut crouch_state,
            &config.movement,
            &config.character,
        );
    }
}
