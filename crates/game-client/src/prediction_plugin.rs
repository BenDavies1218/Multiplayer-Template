//! Prediction sub-plugin — client-side predicted movement and diagnostics.
//!
//! Extracted from [`ClientPlugin`](crate::client::ClientPlugin) so it can be
//! added independently for debugging or selective composition.

use bevy::prelude::*;
use game_networking::movement::update_crouch_collider;
use lightyear::prelude::is_in_rollback;

use crate::diagnostics::{log_despawned_predicted_entities, log_new_predicted_entities};
use crate::movement::handle_character_actions;
use crate::prediction::update_prediction_speed;

/// Runs predicted movement simulation and prediction diagnostics.
pub struct PredictionPlugin;

impl Plugin for PredictionPlugin {
    fn build(&self, app: &mut App) {
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

        // ── Update: diagnostics ─────────────────────────────────────────────
        app.add_systems(
            Update,
            (
                log_new_predicted_entities,
                log_despawned_predicted_entities,
            ),
        );
    }
}
