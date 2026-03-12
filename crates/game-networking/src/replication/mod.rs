//! Replication module
//!
//! Contains shared replication types and systems for entity lifecycle management.
//! Organized into submodules for player and projectile replication.

pub mod player;
pub mod projectile;

use bevy::prelude::*;
use core::time::Duration;

pub use player::CharacterPhysicsBundle;
pub use projectile::shoot_bullet;

/// Marks an entity for automatic despawn after a set lifetime.
#[derive(Component)]
pub struct DespawnAfter {
    pub spawned_at: f32,
    pub lifetime: Duration,
}

pub fn despawn_system(
    mut commands: Commands,
    query: Query<(Entity, &DespawnAfter)>,
    time: Res<Time<Fixed>>,
) {
    for (entity, despawn) in &query {
        if time.elapsed_secs() - despawn.spawned_at >= despawn.lifetime.as_secs_f32() {
            commands.entity(entity).despawn();
        }
    }
}
