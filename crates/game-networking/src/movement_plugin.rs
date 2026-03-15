//! Movement plugin — namespace for shared deterministic movement systems.
//!
//! Both client and server call [`apply_character_movement()`](crate::movement::apply_character_movement)
//! and [`update_crouch_collider()`](crate::movement::update_crouch_collider) directly from their
//! own plugins with their own scheduling. This plugin exists as a logical grouping
//! and future extension point.

use bevy::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, _app: &mut App) {
        // Movement functions are called directly by ClientPlugin/ServerPlugin
        // with their own scheduling contexts. This plugin is a namespace.
    }
}
