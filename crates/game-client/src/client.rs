//! `ClientPlugin` — assembles all client-side systems into the Bevy app.
//!
//! Step 5: Camera-relative movement with jump, sprint, crouch.

use bevy::prelude::*;
use game_networking::movement::update_crouch_collider;
use lightyear::prelude::client::input::InputSystems;
use lightyear::prelude::is_in_rollback;

use crate::character::handle_new_character;
use crate::client_config::ActiveInputDevice;
use crate::diagnostics::{log_despawned_predicted_entities, log_new_predicted_entities};
use crate::input_device::{detect_input_device, rebuild_character_input_map};
use crate::movement::{handle_character_actions, sync_camera_to_character};
use crate::prediction::update_prediction_speed;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        // Insert the active device resource (starts as KeyboardMouse, updated at runtime).
        app.init_resource::<ActiveInputDevice>();

        // Update: detect gamepad connect/disconnect, then rebuild InputMap if needed.
        app.add_systems(
            Update,
            (detect_input_device, rebuild_character_input_map).chain(),
        );

        // ── FixedPreUpdate: write camera yaw into ActionState BEFORE
        //    Lightyear captures it in BufferClientInputs. This ensures
        //    the server receives the same yaw the client uses for movement.
        app.add_systems(
            FixedPreUpdate,
            sync_camera_to_character
                .run_if(not(is_in_rollback))
                .after(InputSystems::WriteClientInputs)
                .before(InputSystems::BufferClientInputs),
        );

        // ── FixedUpdate: simulation ─────────────────────────────────────────
        app.add_systems(
            FixedUpdate,
            (
                update_prediction_speed.run_if(not(is_in_rollback)),
                handle_character_actions,
                update_crouch_collider,
            )
                .chain(),
        );

        // ── Update: entity lifecycle + diagnostics ──────────────────────────
        app.add_systems(
            Update,
            (
                handle_new_character,
                log_new_predicted_entities,
                log_despawned_predicted_entities,
            ),
        );
    }
}
