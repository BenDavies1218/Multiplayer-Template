use bevy::gltf::Gltf;
use bevy::prelude::*;

pub mod events;
mod loader;
mod processor;
pub mod spawn_points;
mod systems;
pub mod zone_debug;
#[allow(clippy::module_inception)]
pub mod zones;

pub use events::*;
pub use spawn_points::SpawnPoints;
pub use zone_debug::{ZoneDebugMesh, ZoneDebugSettings};
pub use zones::*;

/// Configuration for ZonePlugin
#[derive(Resource, Debug, Clone)]
pub struct ZonePluginConfig {
    /// Whether to run collision detection systems (server-only)
    pub enable_detection: bool,
    /// Whether to enable zone debug visualization (viewer-only)
    pub enable_debug: bool,
    /// Whether to process collision_ prefixed nodes into static colliders
    pub load_collision: bool,
    /// Whether to enable collision debug visualization
    pub collision_debug: bool,
}

impl ZonePluginConfig {
    /// Server: load zones + collision, run collision detection, no debug
    pub fn server() -> Self {
        Self {
            enable_detection: true,
            enable_debug: false,
            load_collision: true,
            collision_debug: false,
        }
    }

    /// Client: load collision for physics prediction, no zone detection or debug
    pub fn client() -> Self {
        Self {
            enable_detection: false,
            enable_debug: false,
            load_collision: true,
            collision_debug: false,
        }
    }

    /// World viewer: load zones + collision with debug visualization, no detection
    pub fn viewer() -> Self {
        Self {
            enable_detection: false,
            enable_debug: true,
            load_collision: true,
            collision_debug: true,
        }
    }
}

/// Component for zone mesh loaders (temporary, removed after processing)
#[derive(Component, Debug)]
pub struct ZoneLoader {
    pub handle: Handle<Gltf>,
}

/// Plugin for loading world zones and collision from Blender exports.
///
/// Handles collision geometry, spawn points, death zones, damage zones, and
/// generic triggers — all from a single GLB file using name prefixes.
pub struct ZonePlugin {
    pub config: ZonePluginConfig,
}

impl Plugin for ZonePlugin {
    fn build(&self, app: &mut App) {
        use crate::world::collision_debug;

        app.insert_resource(self.config.clone());

        // Register messages
        app.add_message::<ZoneEnteredEvent>();
        app.add_message::<ZoneExitedEvent>();

        // Loader and processor
        app.add_systems(Startup, loader::load_zone_assets);
        app.add_systems(Update, processor::process_zone_meshes);

        // Server-only: collision detection
        if self.config.enable_detection {
            app.add_systems(
                FixedPostUpdate,
                (systems::detect_zone_collisions, systems::detect_zone_exits),
            );
        }

        // Zone debug visualization (viewer-only)
        if self.config.enable_debug {
            app.init_resource::<ZoneDebugSettings>();
            app.add_systems(Startup, zone_debug::apply_zone_debug_config);
            app.add_systems(
                Update,
                (
                    zone_debug::toggle_zone_debug,
                    zone_debug::update_zone_debug_visibility,
                ),
            );
        }

        // Collision debug visualization (viewer-only)
        if self.config.collision_debug {
            app.init_resource::<crate::world::CollisionDebugSettings>();
            app.add_systems(Startup, collision_debug::apply_debug_config);
            app.add_systems(
                Update,
                (
                    collision_debug::toggle_collision_debug,
                    collision_debug::update_collision_debug_visibility,
                ),
            );
        }
    }
}
