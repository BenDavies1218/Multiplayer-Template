//! Game Networking - Shared networking protocol, replication, and movement
//!
//! This crate contains all shared networking code used by both client and server:
//! - Network protocol definitions (components, actions, replication registration)
//! - Rollback threshold configuration and comparison functions
//! - Shared deterministic movement logic
//! - Replication systems for players and projectiles
//! - Networking configuration and settings

pub mod config;
pub mod movement;
pub mod protocol;
pub mod replication;
pub mod rollback;

// Re-export commonly used items
pub use config::Config;
pub use protocol::*;
pub use replication::{CharacterPhysicsBundle, DespawnAfter};

use bevy::prelude::*;

use avian3d::prelude::*;
use game_core::core_config::GameCoreConfig;
use lightyear::avian3d::plugin::AvianReplicationMode;

use protocol::ProtocolPlugin;
use replication::{despawn_system, projectile::shoot_bullet};

#[derive(Clone)]
pub struct NetworkingPlugin {
    pub config: GameCoreConfig,
}

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        // Initialize rollback thresholds from config
        rollback::init_rollback_config(self.config.rollback_thresholds.clone());

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

        // Gameplay systems
        app.add_systems(FixedUpdate, (shoot_bullet, despawn_system).chain());
    }
}
