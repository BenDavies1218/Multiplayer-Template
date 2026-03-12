//! Client diagnostic systems — log predicted entity lifecycle.
//!
//! Log prefix guide
//! ─────────────────
//! `[NEW-PRED]`   — a Predicted entity just appeared
//! `[PRED-DESPAWN]` — a Predicted entity was removed

use avian3d::prelude::{Collider, LinearVelocity, Position, RigidBody};
use bevy::prelude::*;
use game_core::character::CharacterModelId;
use game_networking::protocol::{
    CameraOrientation, CharacterAction, CharacterMarker, ColorComponent, CrouchState,
    ProjectileMarker,
};
use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::*;

/// Fires once in `Update` when a new `Predicted` entity appears.
/// Logs all relevant components to identify ghost entities.
pub fn log_new_predicted_entities(
    query: Query<Entity, Added<Predicted>>,
    world_query: Query<(
        Option<&Position>,
        Option<&LinearVelocity>,
        Option<&Name>,
        Has<CharacterMarker>,
        Has<ProjectileMarker>,
        Has<Controlled>,
        Has<RigidBody>,
        Has<Collider>,
        Has<ColorComponent>,
        Has<CrouchState>,
        Has<CameraOrientation>,
        Has<ActionState<CharacterAction>>,
        Has<CharacterModelId>,
        Has<Interpolated>,
    )>,
) {
    for entity in &query {
        let Ok((
            pos,
            vel,
            name,
            is_char,
            is_proj,
            is_controlled,
            has_rb,
            has_col,
            has_color,
            has_crouch,
            has_cam,
            has_action,
            has_model,
            has_interp,
        )) = world_query.get(entity)
        else {
            info!("[NEW-PRED] {entity:?} — could not query components");
            continue;
        };

        let pos_str = pos
            .map(|p| format!("({:.3},{:.3},{:.3})", p.x, p.y, p.z))
            .unwrap_or_else(|| "None".to_string());
        let vel_str = vel
            .map(|v| format!("({:.3},{:.3},{:.3})", v.x, v.y, v.z))
            .unwrap_or_else(|| "None".to_string());
        let name_str = name.map(|n| n.as_str()).unwrap_or("unnamed");

        let mut components = Vec::new();
        if is_char {
            components.push("CharacterMarker");
        }
        if is_proj {
            components.push("ProjectileMarker");
        }
        if is_controlled {
            components.push("Controlled");
        }
        if has_rb {
            components.push("RigidBody");
        }
        if has_col {
            components.push("Collider");
        }
        if has_color {
            components.push("ColorComponent");
        }
        if has_crouch {
            components.push("CrouchState");
        }
        if has_cam {
            components.push("CameraOrientation");
        }
        if has_action {
            components.push("ActionState");
        }
        if has_model {
            components.push("CharacterModelId");
        }
        if has_interp {
            components.push("Interpolated");
        }

        info!(
            "[NEW-PRED] {entity:?} name={name_str} pos={pos_str} vel={vel_str}\n  \
             components: [{}]",
            components.join(", ")
        );
    }
}

/// Fires in `Update` when a `Predicted` entity is removed/despawned.
pub fn log_despawned_predicted_entities(
    mut removed: RemovedComponents<Predicted>,
    names: Query<Option<&Name>>,
) {
    for entity in removed.read() {
        let name = names
            .get(entity)
            .ok()
            .flatten()
            .map(|n| n.as_str().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        info!("[PRED-DESPAWN] {entity:?} name={name}");
    }
}
