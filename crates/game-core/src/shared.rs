use bevy::prelude::*;

use crate::protocol::ProtocolPlugin;
use avian3d::prelude::*;
use lightyear::avian3d::plugin::AvianReplicationMode;

pub const CHARACTER_CAPSULE_RADIUS: f32 = 0.5;
pub const CHARACTER_CAPSULE_HEIGHT: f32 = 0.5;

// World asset constants
pub const WORLD_VISUAL_PATH: &str = "models/example_world_visual.glb";
pub const WORLD_COLLISION_PATH: &str = "models/example_world_collision.glb";

#[derive(Bundle)]
pub struct CharacterPhysicsBundle {
    collider: Collider,
    rigid_body: RigidBody,
    lock_axes: LockedAxes,
    friction: Friction,
}

impl Default for CharacterPhysicsBundle {
    fn default() -> Self {
        Self {
            collider: Collider::capsule(CHARACTER_CAPSULE_RADIUS, CHARACTER_CAPSULE_HEIGHT),
            rigid_body: RigidBody::Dynamic,
            lock_axes: LockedAxes::default()
                .lock_rotation_x()
                .lock_rotation_y()
                .lock_rotation_z(),
            friction: Friction::new(0.0).with_combine_rule(CoefficientCombine::Min),
        }
    }
}

#[derive(Clone)]
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        // Neetworking protocols
        app.add_plugins(ProtocolPlugin);

        // types needed for replication and interpolation
        app.register_type::<Transform>();
        app.register_type::<GlobalTransform>();

        // Physics
        app.add_plugins(lightyear::avian3d::plugin::LightyearAvianPlugin {
            replication_mode: AvianReplicationMode::Position,
            ..default()
        });
        app.add_plugins(
            PhysicsPlugins::default()
                .build()
                .disable::<PhysicsTransformPlugin>()
                .disable::<PhysicsInterpolationPlugin>()
                .disable::<IslandPlugin>()
                .disable::<IslandSleepingPlugin>(),
        );

        // WorldPlugin is added separately by each app with the appropriate config
    }
}
