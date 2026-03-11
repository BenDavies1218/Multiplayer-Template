//! Handles new predicted character entities arriving on the client.
//!
//! When a `Predicted` character appears (either the local player or a remote
//! player predicted for us), this module attaches the `InputMap`, physics
//! bundle, and orientation state.

use bevy::prelude::*;
use game_core::GameCoreConfig;
use game_core::core_config::parse_key_code;
use game_core::networking::protocol::{
    CameraOrientation, CharacterAction, CharacterMarker, CrouchState,
};
use game_core::networking::shared::CharacterPhysicsBundle;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::*;

use crate::client_config::{GameClientConfig, parse_gamepad_button};

/// Attach `InputMap`, physics, and orientation components to newly spawned
/// predicted characters.
///
/// Runs in `Update` so it fires once, the frame the entity is added.
pub fn handle_new_character(
    mut commands: Commands,
    mut character_query: Query<
        (Entity, Has<Controlled>),
        (Added<Predicted>, With<CharacterMarker>),
    >,
    client_config: Res<GameClientConfig>,
    core_config: Res<GameCoreConfig>,
) {
    for (entity, is_controlled) in &mut character_query {
        if is_controlled {
            let jump_key =
                parse_key_code(&client_config.input.jump_key).unwrap_or(KeyCode::Space);
            let sprint_key =
                parse_key_code(&client_config.input.sprint_key).unwrap_or(KeyCode::ShiftLeft);
            let crouch_key =
                parse_key_code(&client_config.input.crouch_key).unwrap_or(KeyCode::KeyC);
            let shoot_key =
                parse_key_code(&client_config.input.shoot_key).unwrap_or(KeyCode::KeyQ);
            let jump_gamepad = parse_gamepad_button(&client_config.input.jump_gamepad)
                .unwrap_or(GamepadButton::South);
            let sprint_gamepad = parse_gamepad_button(&client_config.input.sprint_gamepad)
                .unwrap_or(GamepadButton::LeftThumb);
            let crouch_gamepad = parse_gamepad_button(&client_config.input.crouch_gamepad)
                .unwrap_or(GamepadButton::East);

            info!("[character] controlled entity {entity:?} — attaching InputMap");
            commands.entity(entity).insert(
                InputMap::new([(CharacterAction::Jump, jump_key)])
                    .with(CharacterAction::Jump, jump_gamepad)
                    .with(CharacterAction::Sprint, sprint_key)
                    .with(CharacterAction::Sprint, sprint_gamepad)
                    .with(CharacterAction::Crouch, crouch_key)
                    .with(CharacterAction::Crouch, crouch_gamepad)
                    .with(CharacterAction::Shoot, shoot_key)
                    .with(CharacterAction::Shoot, MouseButton::Left)
                    .with_dual_axis(CharacterAction::Move, GamepadStick::LEFT)
                    .with_dual_axis(CharacterAction::Move, VirtualDPad::wasd()),
            );
        } else {
            info!("[character] remote character predicted: {entity:?}");
        }

        info!("[character] {entity:?} — attaching physics + orientation");
        commands.entity(entity).insert((
            CharacterPhysicsBundle::new(&core_config.character),
            CameraOrientation { yaw: 0.0, pitch: 0.0 },
            CrouchState::default(),
        ));
    }
}
