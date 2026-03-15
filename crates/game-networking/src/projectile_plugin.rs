//! Projectile plugin — spawning and lifecycle systems for replicated projectiles.
//!
//! # Systems (FixedUpdate, chained)
//!
//! 1. [`shoot_bullet`](crate::replication::projectile::shoot_bullet) — spawns projectiles
//!    when `CharacterAction::Fire` is pressed
//! 2. [`despawn_system`](crate::replication::despawn_system) — removes entities whose
//!    [`DespawnAfter`](crate::replication::DespawnAfter) lifetime has elapsed

use bevy::prelude::*;

use crate::replication::{despawn_system, projectile::shoot_bullet};

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (shoot_bullet, despawn_system).chain());
    }
}
