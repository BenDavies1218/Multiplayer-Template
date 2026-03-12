//! `ClientPlugin` — assembles all client-side systems into the Bevy app.
//!
//! Each concern lives in its own module:
//!
//! | Module          | Responsibility                                      |
//! |-----------------|-----------------------------------------------------|
//! | `movement`      | Apply inputs → physics via `apply_character_movement` |
//! | `prediction`    | Speed scalar for rollback threshold, rollback logs  |
//! | `character`     | Spawn/configure new predicted character entities    |
//! | `diagnostics`   | Per-tick state logging (position, velocity, diffs)  |
//!
//! # FixedUpdate tick order
//!
//! ```text
//! update_prediction_speed  (not(is_in_rollback)) — update CURRENT_SPEED before threshold check
//! log_rollback_tick        (is_in_rollback)       — confirm rollback detection is working
//! track_velocity_changes                          — detect velocity jumps / rollback signatures
//! sync_camera_to_character (not(is_in_rollback))  — write camera yaw into ActionState
//! handle_character_actions                        — apply movement forces
//! update_crouch_collider                          — resize collider when CrouchState changes
//!    ↓ (separate, after update_crouch_collider)
//! log_all_predicted_state                         — snapshot position/vel vs server
//! ```

use bevy::prelude::*;
use game_core::movement::update_crouch_collider;
use lightyear::prelude::is_in_rollback;

use crate::character::handle_new_character;
use crate::client_config::ActiveInputDevice;
use crate::input_device::{detect_input_device, rebuild_character_input_map};
use crate::diagnostics::{
    log_all_predicted_state, log_despawned_predicted_entities, log_new_predicted_entities,
};
use crate::movement::{handle_character_actions, sync_camera_to_character};
use crate::prediction::{log_rollback_tick, track_velocity_changes, update_prediction_speed};

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

        // ── FixedUpdate: main simulation chain ──────────────────────────────
        app.add_systems(
            FixedUpdate,
            (
                // Update CURRENT_SPEED *before* any comparison; skip during rollback
                // so historical velocities don't corrupt the threshold.
                update_prediction_speed.run_if(not(is_in_rollback)),
                // Confirm rollback re-simulation is firing (warn-level so it's visible)
                log_rollback_tick.run_if(is_in_rollback),
                // Detect velocity spikes caused by rollback state restoration
                track_velocity_changes,
                // Sync camera yaw into ActionState only on normal ticks
                sync_camera_to_character.run_if(not(is_in_rollback)),
                // Apply inputs → forces
                handle_character_actions,
                // Resize collider when crouch state changes
                update_crouch_collider,
            )
                .chain(),
        );

        // ── FixedUpdate: end-of-tick diagnostics ────────────────────────────
        // Runs after `update_crouch_collider` to capture final state.
        app.add_systems(
            FixedUpdate,
            log_all_predicted_state.after(update_crouch_collider),
        );

        // ── Update: entity lifecycle diagnostics ────────────────────────────
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
