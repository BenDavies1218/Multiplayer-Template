use bevy::gltf::Gltf;
use bevy::prelude::*;

use crate::utils::config_hot_reload::ConfigWatchExt;
use crate::utils::config_loader::load_config;

pub mod actions;
pub mod debug;
pub mod events;
pub mod light_effects;
mod loader;
mod processor;
pub mod triggers;
pub mod types;

pub use debug::{DynamicDebugMesh, DynamicDebugSettings};
pub use events::*;
pub use types::*;

/// Configuration for DynamicPlugin.
#[derive(Resource, Debug, Clone)]
pub struct DynamicPluginConfig {
    /// Whether to run trigger detection systems (server-only).
    pub enable_triggers: bool,
    /// Whether to enable visual action execution (client-only).
    pub enable_visuals: bool,
    /// Whether to enable debug visualization (viewer-only).
    pub enable_debug: bool,
}

impl DynamicPluginConfig {
    /// Server: load dynamic objects and run trigger detection.
    pub fn server() -> Self {
        Self {
            enable_triggers: true,
            enable_visuals: false,
            enable_debug: false,
        }
    }

    /// Client: load dynamic objects for visual rendering.
    pub fn client() -> Self {
        Self {
            enable_triggers: false,
            enable_visuals: true,
            enable_debug: false,
        }
    }

    /// World viewer: load with debug visualization.
    pub fn viewer() -> Self {
        Self {
            enable_triggers: false,
            enable_visuals: false,
            enable_debug: true,
        }
    }
}

/// Component for dynamic object loaders (temporary, removed after processing).
#[derive(Component, Debug)]
pub struct DynamicLoader {
    pub handle: Handle<Gltf>,
}

/// Plugin for loading and managing dynamic interactable objects from Blender exports.
///
/// Dynamic objects are defined in a separate GLB file (`world_dynamic.glb`) with
/// behavior driven by Blender custom properties (GltfExtras).
///
/// - Server loads objects and runs trigger detection + state actions.
/// - Client loads objects for visual action execution (animations, lights, text).
/// - Viewer loads objects with debug visualization.
pub struct DynamicPlugin {
    pub config: DynamicPluginConfig,
}

impl Plugin for DynamicPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config.clone());

        let dynamic_objects_config: DynamicObjectsConfig =
            load_config("dynamic_objects_config.json");
        app.insert_resource(dynamic_objects_config);
        app.watch_config::<DynamicObjectsConfig>("dynamic_objects_config.json");

        // Register events
        app.add_message::<DynamicTriggerEvent>();
        app.add_message::<DynamicActionEvent>();

        // Loader and processor (all modes)
        app.add_systems(Startup, loader::load_dynamic_assets);
        app.add_systems(Update, processor::process_dynamic_objects);

        // Server-only: trigger detection and action execution
        if self.config.enable_triggers {
            app.add_systems(
                FixedPostUpdate,
                (
                    triggers::detect_enter_exit_triggers,
                    triggers::detect_spawn_triggers,
                    triggers::detect_interact_triggers,
                    triggers::dispatch_trigger_actions,
                    actions::execute_state_actions,
                )
                    .chain(),
            );
        }

        // Light effects (client + viewer)
        if self.config.enable_visuals || self.config.enable_debug {
            app.add_systems(Update, light_effects::tick_light_effects);
        }

        // Debug visualization (viewer-only)
        if self.config.enable_debug {
            app.init_resource::<DynamicDebugSettings>();
            app.add_systems(Startup, debug::apply_dynamic_debug_config);
            app.add_systems(
                Update,
                (
                    debug::toggle_dynamic_debug,
                    debug::update_dynamic_debug_visibility,
                ),
            );
        }
    }
}
