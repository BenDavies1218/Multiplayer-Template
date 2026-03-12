//! Runtime input device detection and InputMap rebuilding.
//!
//! Two systems run every `Update` tick in order:
//!
//! 1. `detect_input_device` — reacts to gamepad connect/disconnect events and
//!    updates `ActiveInputDevice` according to the config preference.
//! 2. `rebuild_character_input_map` — watches `ActiveInputDevice` for changes
//!    and rebuilds the `InputMap` on all character entities.

use bevy::input::gamepad::GamepadConnectionEvent;
use bevy::prelude::*;
use bevy::prelude::MessageReader;
use game_core::networking::protocol::{CharacterAction, CharacterMarker};
use leafwing_input_manager::prelude::*;

use crate::client_config::{ActiveInputDevice, GameClientConfig, InputDevice, ResolvedDevice};
use crate::character::{build_gamepad_input_map, build_keyboard_input_map};

/// Resolves `GameClientConfig.input.active_device` against the set of
/// currently connected gamepads, updating `ActiveInputDevice`.
///
/// Runs on every `GamepadConnectionEvent` frame.
pub fn detect_input_device(
    mut events: MessageReader<GamepadConnectionEvent>,
    gamepads: Query<Entity, With<Gamepad>>,
    config: Res<GameClientConfig>,
    mut active: ResMut<ActiveInputDevice>,
) {
    // Only do work when there are connection events this frame.
    if events.is_empty() {
        return;
    }
    events.clear();

    // Re-evaluate from the current connected set.
    let first_gamepad = gamepads.iter().next();

    let resolved = match config.input.active_device {
        InputDevice::KeyboardMouse => ResolvedDevice::KeyboardMouse,
        InputDevice::Gamepad | InputDevice::Auto => {
            if first_gamepad.is_some() {
                ResolvedDevice::Gamepad
            } else {
                ResolvedDevice::KeyboardMouse
            }
        }
    };

    active.device = resolved;
    active.gamepad = first_gamepad;

    info!(
        "[input] resolved device: {:?} (gamepad entity: {:?})",
        active.device, active.gamepad
    );
}

/// Rebuilds the `InputMap` on all character entities when `ActiveInputDevice`
/// changes.
///
/// Uses a `Local` to remember the last resolved device and only acts on
/// transitions.
pub fn rebuild_character_input_map(
    active: Res<ActiveInputDevice>,
    config: Res<GameClientConfig>,
    mut characters: Query<&mut InputMap<CharacterAction>, With<CharacterMarker>>,
    mut last: Local<Option<ResolvedDevice>>,
) {
    let changed = match *last {
        None => true,
        Some(prev) => prev != active.device,
    };
    if !changed && !config.is_changed() {
        return;
    }

    *last = Some(active.device);

    let new_map = match active.device {
        ResolvedDevice::KeyboardMouse => build_keyboard_input_map(&config),
        ResolvedDevice::Gamepad => build_gamepad_input_map(&config),
    };

    for mut map in &mut characters {
        *map = new_map.clone();
    }

    info!("[input] InputMap rebuilt for device: {:?}", active.device);
}
