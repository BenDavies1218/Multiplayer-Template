//! Client-side movement systems.
//!
//! `handle_character_actions` drives physics on the local predicted entity.
//! `sync_camera_to_character` writes the current camera yaw/pitch into the
//! entity's `ActionState` so movement is always camera-relative.
//!
//! `sync_camera_to_character` is skipped during rollback re-simulation because
//! Lightyear restores the historical `ActionState` (including the Look axis)
//! for each replayed tick. Overwriting it with the *current* camera angle
//! would diverge from the server's recorded input.

use avian3d::prelude::*;
use bevy::prelude::*;
use game_camera::GameCamera;
use game_core::GameCoreConfig;
use game_networking::movement::apply_character_movement;
use game_networking::protocol::{CharacterAction, CharacterMarker, CrouchState};
use leafwing_input_manager::prelude::*;
use lightyear::prelude::*;

/// Process `CharacterAction` inputs and drive physics via `apply_character_movement`.
///
/// Runs every `FixedUpdate` tick on all `Predicted` character entities,
/// including during rollback re-simulation.
pub fn handle_character_actions(
    spatial_query: SpatialQuery,
    mut query: Query<
        (
            Entity,
            &ActionState<CharacterAction>,
            &Position,
            &mut LinearVelocity,
            &mut CrouchState,
        ),
        With<Predicted>,
    >,
    config: Res<GameCoreConfig>,
) {
    for (entity, action_state, position, mut linear_velocity, mut crouch_state) in &mut query {
        let camera_yaw = action_state.axis_pair(&CharacterAction::Look).x;
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

/// Copy the current camera yaw/pitch into the character's `ActionState`.
///
/// Skipped during rollback — Lightyear restores the historical `ActionState`
/// including the Look axis before each re-simulated tick.
pub fn sync_camera_to_character(
    camera_query: Query<&GameCamera>,
    mut character_query: Query<
        &mut ActionState<CharacterAction>,
        (With<CharacterMarker>, With<Predicted>, With<Controlled>),
    >,
) {
    let Ok(game_camera) = camera_query.single() else {
        return;
    };

    for mut action_state in &mut character_query {
        action_state.set_axis_pair(
            &CharacterAction::Look,
            Vec2::new(game_camera.yaw, game_camera.pitch),
        );
    }
}
