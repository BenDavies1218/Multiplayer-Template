//! Player replication
//!
//! Physics bundle and components used when spawning replicated player characters.

use avian3d::prelude::*;
use bevy::prelude::*;

#[derive(Bundle)]
pub struct CharacterPhysicsBundle {
    collider: Collider,
    rigid_body: RigidBody,
    lock_axes: LockedAxes,
    friction: Friction,
}

impl CharacterPhysicsBundle {
    /// Create a physics bundle using values from `CharacterConfig`.
    pub fn new(character: &game_core::simulation_config::CharacterConfig) -> Self {
        Self {
            collider: Collider::capsule(character.capsule_radius, character.capsule_height),
            rigid_body: RigidBody::Dynamic,
            lock_axes: LockedAxes::default()
                .lock_rotation_x()
                .lock_rotation_y()
                .lock_rotation_z(),
            friction: Friction::new(0.0).with_combine_rule(CoefficientCombine::Min),
        }
    }
}
