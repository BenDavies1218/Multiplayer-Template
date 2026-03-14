use bevy::gltf::Gltf;
use bevy::prelude::*;

use super::DynamicLoader;
use crate::core_config::GameCoreConfig;

/// Load dynamic object assets at startup.
///
/// Loads the dynamic GLB file which contains nodes with custom properties
/// defining triggers, actions, and state for interactable objects.
pub fn load_dynamic_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<GameCoreConfig>,
) {
    let handle: Handle<Gltf> = asset_server.load(config.world_assets.dynamic_path.clone());
    commands.spawn(DynamicLoader {
        handle: handle.clone(),
    });
    info!(
        "Loading dynamic objects from {}",
        config.world_assets.dynamic_path
    );
}
