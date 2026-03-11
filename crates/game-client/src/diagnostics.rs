//! Client diagnostic systems — log predicted entity state each tick.
//!
//! These systems are intentionally verbose; gate them with `RUST_LOG` filters
//! or comment out registrations in `ClientPlugin` when not debugging.
//!
//! Log prefix guide
//! ─────────────────
//! `[NEW-PRED]`   — a Predicted entity just appeared
//! `[PRED-DESPAWN]` — a Predicted entity was removed
//! `[PRED-STATE]` — end-of-tick snapshot: position, velocity, server diff

use avian3d::prelude::{LinearVelocity, Position};
use bevy::prelude::*;
use game_core::networking::protocol::{CharacterMarker, ProjectileMarker};
use lightyear::prelude::*;

/// Fires once in `Update` when a new `Predicted` entity appears.
pub fn log_new_predicted_entities(
    query: Query<
        (
            Entity,
            Option<&Position>,
            Option<&LinearVelocity>,
            Has<CharacterMarker>,
            Has<ProjectileMarker>,
            Has<Controlled>,
        ),
        Added<Predicted>,
    >,
) {
    for (entity, pos, vel, is_char, is_proj, is_controlled) in &query {
        let pos_str = pos
            .map(|p| format!("({:.3},{:.3},{:.3})", p.x, p.y, p.z))
            .unwrap_or_else(|| "None".to_string());
        let vel_str = vel
            .map(|v| format!("({:.3},{:.3},{:.3})", v.x, v.y, v.z))
            .unwrap_or_else(|| "None".to_string());
        info!(
            "[NEW-PRED] {entity:?} pos={pos_str} vel={vel_str} \
             char={is_char} proj={is_proj} controlled={is_controlled}"
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

/// Runs at the end of each `FixedUpdate` tick (after movement + physics).
///
/// Logs predicted position/velocity alongside the last server-confirmed values
/// so you can see exactly how much the client has diverged from the server.
/// `pos_diff_xz` is the horizontal distance — the same metric used by
/// `position_should_rollback`.
pub fn log_all_predicted_state(
    timeline: Res<LocalTimeline>,
    query: Query<
        (
            Entity,
            &Position,
            &LinearVelocity,
            Option<&Confirmed<Position>>,
            Option<&Confirmed<LinearVelocity>>,
            Has<CharacterMarker>,
            Has<ProjectileMarker>,
            Has<Controlled>,
        ),
        With<Predicted>,
    >,
) {
    let tick = timeline.tick();

    for (entity, pos, vel, conf_pos, conf_vel, is_char, is_proj, is_controlled) in &query {
        let kind = if is_char {
            if is_controlled { "char(local)" } else { "char(remote)" }
        } else if is_proj {
            "projectile"
        } else {
            "unknown"
        };

        let conf_pos_str = conf_pos
            .map(|cp| format!("({:.3},{:.3},{:.3})", cp.0.x, cp.0.y, cp.0.z))
            .unwrap_or_else(|| "n/a".to_string());
        let conf_vel_str = conf_vel
            .map(|cv| format!("({:.3},{:.3},{:.3})", cv.0.x, cv.0.y, cv.0.z))
            .unwrap_or_else(|| "n/a".to_string());

        // XZ divergence — the metric position_should_rollback uses
        let pos_diff_xz = conf_pos.map(|cp| {
            let d = pos.0 - cp.0.0;
            Vec2::new(d.x, d.z).length()
        });
        let vel_diff = conf_vel.map(|cv| (vel.0 - cv.0.0).length());

        debug!(
            "[PRED-STATE] tick={tick:?} {entity:?} kind={kind}\n  \
             pred_pos=({:.3},{:.3},{:.3}) conf_pos={conf_pos_str} pos_diff_xz={}\n  \
             pred_vel=({:.3},{:.3},{:.3}) conf_vel={conf_vel_str} vel_diff={}",
            pos.x, pos.y, pos.z,
            pos_diff_xz
                .map(|d| format!("{d:.4}m"))
                .unwrap_or_else(|| "n/a".to_string()),
            vel.x, vel.y, vel.z,
            vel_diff
                .map(|d| format!("{d:.4}m/s"))
                .unwrap_or_else(|| "n/a".to_string()),
        );
    }
}
