//! Game Networking — Shared networking protocol, replication, and movement
//!
//! This crate contains all shared networking code used by both client and server:
//! - Network protocol definitions (components, actions, replication registration)
//! - Rollback threshold configuration and comparison functions
//! - Shared deterministic movement logic
//! - Replication systems for players and projectiles
//! - Networking configuration and settings
//!
//! # Sub-Plugin Architecture
//!
//! [`NetworkingPlugin`] is a convenience wrapper that composes these sub-plugins:
//!
//! | Plugin | Responsibility |
//! |--------|---------------|
//! | [`ProtocolPlugin`] | Component registration, prediction config, input plugin |
//! | [`RollbackPlugin`] | Threshold initialization from [`RollbackConfig`](game_core::core_config::RollbackConfig) |
//! | [`ProjectilePlugin`] | Bullet spawning + timed despawn (FixedUpdate) |
//! | [`MovementPlugin`] | Shared movement function namespace (systems scheduled by consumers) |
//!
//! Apps that use [`NetworkingPlugin`] get all of the above plus physics setup
//! (Avian3d + Lightyear replication). For fine-grained control, apps can add
//! individual sub-plugins directly.

pub mod config;
pub mod movement;
pub mod movement_plugin;
pub mod projectile_plugin;
pub mod protocol;
pub mod replication;
pub mod rollback;
pub mod rollback_plugin;

// Re-export commonly used items
pub use config::Config;
pub use protocol::*;
pub use replication::{CharacterPhysicsBundle, DespawnAfter};

// Re-export sub-plugins for direct use
pub use movement_plugin::MovementPlugin;
pub use projectile_plugin::ProjectilePlugin;
pub use protocol::ProtocolPlugin;
pub use rollback_plugin::RollbackPlugin;

use bevy::prelude::*;

use avian3d::prelude::*;
use game_core::performance_config::GamePerformanceConfig;
use game_core::simulation_config::GameSimulationConfig;
use lightyear::avian3d::plugin::AvianReplicationMode;

#[derive(Clone)]
pub struct NetworkingPlugin {
    pub simulation: GameSimulationConfig,
    pub performance: GamePerformanceConfig,
}

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        // Rollback threshold initialization
        app.add_plugins(RollbackPlugin {
            config: self.performance.rollback_thresholds.clone(),
        });

        // Insert config resources for other systems to use
        app.insert_resource(self.simulation.clone());
        app.insert_resource(self.performance.clone());

        // Networking protocols
        app.add_plugins(ProtocolPlugin);

        // Projectile spawning and despawn
        app.add_plugins(ProjectilePlugin);

        // Movement namespace (systems scheduled by client/server)
        app.add_plugins(MovementPlugin);

        // Types needed for replication and interpolation
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
    }
}
