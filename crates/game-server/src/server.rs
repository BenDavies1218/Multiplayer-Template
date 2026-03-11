//! `ServerPlugin` — assembles all server-side systems into the Bevy app.
//!
//! Each concern lives in its own module:
//!
//! | Module        | Responsibility                                        |
//! |---------------|-------------------------------------------------------|
//! | `movement`    | Authoritative movement — reads inputs, drives physics |
//! | `spawning`    | Client connect observers, character spawning          |
//! | `diagnostics` | Per-tick state logging for comparison with client     |
//!
//! # FixedUpdate tick order
//!
//! ```text
//! handle_character_actions  — apply movement forces (authoritative)
//! update_crouch_collider    — resize collider on CrouchState change
//! log_server_character_state — snapshot pos/vel for debugging
//! ```

use bevy::prelude::*;
use game_core::movement::update_crouch_collider;

use crate::diagnostics::log_server_character_state;
use crate::movement::handle_character_actions;
use crate::spawning::{handle_connected, handle_new_client};

#[derive(Clone)]
pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                handle_character_actions,
                update_crouch_collider,
                log_server_character_state,
            )
                .chain(),
        );

        app.add_observer(handle_new_client);
        app.add_observer(handle_connected);
    }
}
