//! Handles new predicted character entities arriving on the client.
//!
//! When a `Predicted` character appears (either the local player or a remote
//! player predicted for us), this module attaches the `InputMap`, physics
//! bundle, and orientation state.

use bevy::prelude::*;
use game_core::GameSimulationConfig;
use game_core::core_config::parse_key_code;
use game_networking::protocol::{CameraOrientation, CharacterAction, CharacterMarker, CrouchState};
use game_networking::replication::CharacterPhysicsBundle;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::*;

use crate::client_config::{ActiveInputDevice, GameClientConfig, ResolvedDevice};

/// Build a keyboard+mouse `InputMap` from config.
pub fn build_keyboard_input_map(config: &GameClientConfig) -> InputMap<CharacterAction> {
    let kb = &config.input.keyboard;

    let move_up = parse_key_code(&kb.move_up).unwrap_or(KeyCode::KeyW);
    let move_down = parse_key_code(&kb.move_down).unwrap_or(KeyCode::KeyS);
    let move_left = parse_key_code(&kb.move_left).unwrap_or(KeyCode::KeyA);
    let move_right = parse_key_code(&kb.move_right).unwrap_or(KeyCode::KeyD);
    let jump = parse_key_code(&kb.jump).unwrap_or(KeyCode::Space);
    let sprint = parse_key_code(&kb.sprint).unwrap_or(KeyCode::ShiftLeft);
    let crouch = parse_key_code(&kb.crouch).unwrap_or(KeyCode::ControlLeft);
    let prone = parse_key_code(&kb.prone).unwrap_or(KeyCode::KeyC);
    let reload = parse_key_code(&kb.reload).unwrap_or(KeyCode::KeyR);
    let interact = parse_key_code(&kb.interact).unwrap_or(KeyCode::KeyE);
    let lethal = parse_key_code(&kb.lethal_equipment).unwrap_or(KeyCode::KeyQ);
    let melee = parse_key_code(&kb.melee).unwrap_or(KeyCode::KeyV);
    let inspect = parse_key_code(&kb.weapon_inspect).unwrap_or(KeyCode::KeyG);
    let armor = parse_key_code(&kb.armor_plate).unwrap_or(KeyCode::KeyF);
    let alt_fire = parse_key_code(&kb.alternate_fire).unwrap_or(KeyCode::KeyB);
    let primary = parse_key_code(&kb.primary_weapon).unwrap_or(KeyCode::Digit1);
    let secondary = parse_key_code(&kb.secondary_weapon).unwrap_or(KeyCode::Digit2);
    let ks1 = parse_key_code(&kb.killstreak1).unwrap_or(KeyCode::Digit3);
    let ks2 = parse_key_code(&kb.killstreak2).unwrap_or(KeyCode::Digit4);
    let ks3 = parse_key_code(&kb.killstreak3).unwrap_or(KeyCode::Digit5);
    let fu = parse_key_code(&kb.field_upgrade).unwrap_or(KeyCode::KeyX);
    let ping = parse_key_code(&kb.ping).unwrap_or(KeyCode::KeyZ);
    let ptt = parse_key_code(&kb.push_to_talk).unwrap_or(KeyCode::CapsLock);
    let scoreboard = parse_key_code(&kb.scoreboard).unwrap_or(KeyCode::Tab);
    let map_key = parse_key_code(&kb.map).unwrap_or(KeyCode::KeyM);
    let inventory = parse_key_code(&kb.inventory).unwrap_or(KeyCode::KeyI);
    let pause = parse_key_code(&kb.pause).unwrap_or(KeyCode::Escape);
    let night_vis = parse_key_code(&kb.night_vision).unwrap_or(KeyCode::KeyN);
    let text_chat = parse_key_code(&kb.text_chat).unwrap_or(KeyCode::KeyT);
    let team_chat = parse_key_code(&kb.team_chat).unwrap_or(KeyCode::KeyY);
    let g1 = parse_key_code(&kb.gesture1).unwrap_or(KeyCode::F1);
    let g2 = parse_key_code(&kb.gesture2).unwrap_or(KeyCode::F2);
    let g3 = parse_key_code(&kb.gesture3).unwrap_or(KeyCode::F3);
    let g4 = parse_key_code(&kb.gesture4).unwrap_or(KeyCode::F4);

    InputMap::new([(CharacterAction::Jump, jump)])
        .with(CharacterAction::Sprint, sprint)
        .with(CharacterAction::Crouch, crouch)
        .with(CharacterAction::Prone, prone)
        .with(CharacterAction::MountLedge, jump)
        .with(CharacterAction::Fire, MouseButton::Left)
        .with(CharacterAction::AimDownSights, MouseButton::Right)
        .with(CharacterAction::TacticalEquipment, MouseButton::Middle)
        .with(CharacterAction::Reload, reload)
        .with(CharacterAction::PrimaryWeapon, primary)
        .with(CharacterAction::SecondaryWeapon, secondary)
        .with(CharacterAction::Interact, interact)
        .with(CharacterAction::LethalEquipment, lethal)
        .with(CharacterAction::Melee, melee)
        .with(CharacterAction::WeaponInspect, inspect)
        .with(CharacterAction::ArmorPlate, armor)
        .with(CharacterAction::AlternateFire, alt_fire)
        .with(CharacterAction::Killstreak1, ks1)
        .with(CharacterAction::Killstreak2, ks2)
        .with(CharacterAction::Killstreak3, ks3)
        .with(CharacterAction::FieldUpgrade, fu)
        .with(CharacterAction::Ping, ping)
        .with(CharacterAction::PushToTalk, ptt)
        .with(CharacterAction::TextChat, text_chat)
        .with(CharacterAction::TeamChat, team_chat)
        .with(CharacterAction::Gesture1, g1)
        .with(CharacterAction::Gesture2, g2)
        .with(CharacterAction::Gesture3, g3)
        .with(CharacterAction::Gesture4, g4)
        .with(CharacterAction::Scoreboard, scoreboard)
        .with(CharacterAction::Map, map_key)
        .with(CharacterAction::Inventory, inventory)
        .with(CharacterAction::Pause, pause)
        .with(CharacterAction::NightVision, night_vis)
        .with_dual_axis(
            CharacterAction::Move,
            VirtualDPad::new(move_up, move_down, move_left, move_right),
        )
        .with_dual_axis(CharacterAction::Look, MouseMove::default())
}

/// Build a gamepad `InputMap` from config.
pub fn build_gamepad_input_map(config: &GameClientConfig) -> InputMap<CharacterAction> {
    use crate::client_config::{parse_gamepad_button, parse_gamepad_stick};

    let gp = &config.input.gamepad;

    let jump = parse_gamepad_button(&gp.jump).unwrap_or(GamepadButton::South);
    let fire = parse_gamepad_button(&gp.fire).unwrap_or(GamepadButton::RightTrigger);
    let ads = parse_gamepad_button(&gp.aim_down_sights).unwrap_or(GamepadButton::LeftTrigger);
    let reload = parse_gamepad_button(&gp.reload).unwrap_or(GamepadButton::West);
    let melee = parse_gamepad_button(&gp.melee).unwrap_or(GamepadButton::East);
    let sprint = parse_gamepad_button(&gp.sprint).unwrap_or(GamepadButton::LeftThumb);
    let crouch = parse_gamepad_button(&gp.crouch).unwrap_or(GamepadButton::RightThumb);
    let lethal = parse_gamepad_button(&gp.lethal_equipment).unwrap_or(GamepadButton::LeftTrigger2);
    let tactical =
        parse_gamepad_button(&gp.tactical_equipment).unwrap_or(GamepadButton::RightTrigger2);
    let ks1 = parse_gamepad_button(&gp.killstreak1).unwrap_or(GamepadButton::DPadRight);
    let ks2 = parse_gamepad_button(&gp.killstreak2).unwrap_or(GamepadButton::DPadRight);
    let ks3 = parse_gamepad_button(&gp.killstreak3).unwrap_or(GamepadButton::DPadRight);
    let fu = parse_gamepad_button(&gp.field_upgrade).unwrap_or(GamepadButton::DPadLeft);
    let ping = parse_gamepad_button(&gp.ping).unwrap_or(GamepadButton::DPadLeft);
    let armor = parse_gamepad_button(&gp.armor_plate).unwrap_or(GamepadButton::DPadUp);
    let night = parse_gamepad_button(&gp.night_vision).unwrap_or(GamepadButton::DPadDown);
    let score = parse_gamepad_button(&gp.scoreboard).unwrap_or(GamepadButton::Select);
    let pause = parse_gamepad_button(&gp.pause).unwrap_or(GamepadButton::Start);

    let _move_stick = parse_gamepad_stick(&gp.move_stick).unwrap_or(GamepadStick::LEFT);
    let _look_stick = parse_gamepad_stick(&gp.look_stick).unwrap_or(GamepadStick::RIGHT);

    InputMap::new([(CharacterAction::Jump, jump)])
        .with(CharacterAction::Sprint, sprint)
        .with(CharacterAction::Crouch, crouch)
        .with(CharacterAction::MountLedge, jump)
        .with(CharacterAction::Fire, fire)
        .with(CharacterAction::AimDownSights, ads)
        .with(CharacterAction::Reload, reload)
        .with(CharacterAction::Melee, melee)
        .with(CharacterAction::LethalEquipment, lethal)
        .with(CharacterAction::TacticalEquipment, tactical)
        .with(CharacterAction::Killstreak1, ks1)
        .with(CharacterAction::Killstreak2, ks2)
        .with(CharacterAction::Killstreak3, ks3)
        .with(CharacterAction::FieldUpgrade, fu)
        .with(CharacterAction::Ping, ping)
        .with(CharacterAction::ArmorPlate, armor)
        .with(CharacterAction::NightVision, night)
        .with(CharacterAction::Scoreboard, score)
        .with(CharacterAction::Pause, pause)
        .with_dual_axis(CharacterAction::Move, GamepadStick::LEFT)
        .with_dual_axis(CharacterAction::Look, GamepadStick::RIGHT)
}

/// Attach `InputMap`, physics, and orientation components to newly spawned
/// predicted characters.
///
/// Runs in `Update` so it fires once, the frame the entity is added.
#[allow(clippy::type_complexity)]
pub fn handle_new_character(
    mut commands: Commands,
    mut character_query: Query<
        (Entity, Has<Controlled>),
        (Added<Predicted>, With<CharacterMarker>),
    >,
    client_config: Res<GameClientConfig>,
    sim_config: Res<GameSimulationConfig>,
    active_device: Res<ActiveInputDevice>,
) {
    for (entity, is_controlled) in &mut character_query {
        if is_controlled {
            info!("[character] controlled entity {entity:?} — attaching InputMap");

            let input_map = match active_device.device {
                ResolvedDevice::KeyboardMouse => build_keyboard_input_map(&client_config),
                ResolvedDevice::Gamepad => build_gamepad_input_map(&client_config),
            };

            commands.entity(entity).insert(input_map);
        } else {
            info!("[character] remote character predicted: {entity:?}");
        }

        info!("[character] {entity:?} — attaching physics + orientation");
        commands.entity(entity).insert((
            CharacterPhysicsBundle::new(&sim_config.character),
            CameraOrientation {
                yaw: 0.0,
                pitch: 0.0,
            },
            CrouchState::default(),
        ));
    }
}
