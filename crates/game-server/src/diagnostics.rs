//! Server diagnostic systems.
//!
//! Compare `[SRV-STATE]` output against the client's `[PRED-STATE]` output to
//! identify where the simulation diverges.  If the two are consistently
//! different at the same tick, that divergence will trigger a client rollback.

use avian3d::prelude::*;
use bevy::prelude::*;
use game_core::networking::protocol::CharacterMarker;
use lightyear::core::timeline::LocalTimeline;

/// Log authoritative server state for every character each `FixedUpdate` tick.
pub fn log_server_character_state(
    timeline: Res<LocalTimeline>,
    query: Query<(Entity, &Position, &LinearVelocity), With<CharacterMarker>>,
) {
    let tick = timeline.tick();
    for (entity, pos, vel) in &query {
        let horiz_speed = Vec2::new(vel.x, vel.z).length();
        debug!(
            "[SRV-STATE] tick={tick:?} {entity:?} \
             pos=({:.3},{:.3},{:.3}) vel=({:.3},{:.3},{:.3}) speed={horiz_speed:.3}",
            pos.x, pos.y, pos.z,
            vel.x, vel.y, vel.z,
        );
    }
}
