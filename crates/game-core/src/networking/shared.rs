use bevy::prelude::*;

use super::protocol::ProtocolPlugin;
use avian3d::prelude::*;
use lightyear::avian3d::plugin::AvianReplicationMode;

use crate::core_config::GameCoreConfig;

#[derive(Bundle)]
pub struct CharacterPhysicsBundle {
    collider: Collider,
    rigid_body: RigidBody,
    lock_axes: LockedAxes,
    friction: Friction,
}

impl CharacterPhysicsBundle {
    /// Create a physics bundle using values from `CharacterConfig`.
    pub fn new(character: &crate::core_config::CharacterConfig) -> Self {
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

#[derive(Clone)]
pub struct SharedPlugin {
    pub config: GameCoreConfig,
}

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        // Initialize rollback thresholds from config
        crate::networking::protocol::init_rollback_config(self.config.rollback_thresholds.clone());

        // Insert GameCoreConfig as a resource for other systems to use
        app.insert_resource(self.config.clone());

        // Networking protocols
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
