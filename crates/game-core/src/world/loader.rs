use bevy::prelude::*;
use bevy::scene::SceneRoot;

use super::{WorldAssets, WorldPluginConfig, WorldVisual};
use crate::world_config::GameWorldConfig;

/// Load world visual assets at startup
pub fn load_world_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    plugin_config: Res<WorldPluginConfig>,
    config: Res<GameWorldConfig>,
) {
    let mut visual_handle = None;

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

    // Store handles in resource for later access
    commands.insert_resource(WorldAssets {
        visual: visual_handle,
    });
}
