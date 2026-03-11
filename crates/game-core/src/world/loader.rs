use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy::scene::SceneRoot;

use super::{WorldAssets, WorldCollisionLoader, WorldPluginConfig, WorldVisual};
use crate::core_config::GameCoreConfig;

/// Load world assets at startup
///
/// To use this system, place your world files in:
/// - `assets/models/example_world_visual.glb` - High-poly visual scene
/// - `assets/models/example_world_collision.glb` - Low-poly collision scene
///
pub fn load_world_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    plugin_config: Res<WorldPluginConfig>,
    config: Res<GameCoreConfig>,
) {
    let mut visual_handle = None;
    let mut collision_handle = None;

    // Load visual scene if enabled
    if plugin_config.load_visual {
        let handle = asset_server.load(format!("{}#Scene0", config.world_assets.visual_path));

        // Spawn the visual scene WITHOUT ColliderConstructor to prevent auto-collider generation
        commands.spawn((
            SceneRoot(handle.clone()),
            WorldVisual,
            Transform::default(),
            GlobalTransform::default(),
        ));

        visual_handle = Some(handle);
        info!(
            "Loading world visual from {}",
            config.world_assets.visual_path
        );
    }

    // Load collision mesh if enabled
    if plugin_config.load_collision {
        let handle: Handle<Gltf> = asset_server.load(config.world_assets.collision_path.clone());
        commands.spawn(WorldCollisionLoader {
            handle: handle.clone(),
        });

        collision_handle = Some(handle);
        info!(
            "Loading world collision from {}",
            config.world_assets.collision_path
        );
    }

    // Store handles in resource for later access
    commands.insert_resource(WorldAssets {
        visual: visual_handle,
        collision: collision_handle,
    });
}
