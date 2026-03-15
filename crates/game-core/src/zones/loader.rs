use bevy::gltf::Gltf;
use bevy::prelude::*;

use super::ZoneLoader;
use crate::world_config::GameWorldConfig;

/// Load zone assets at startup.
///
/// Loads the zones GLB file which contains named nodes for spawn points,
/// death zones, damage zones, and generic zone triggers.
pub fn load_zone_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<GameWorldConfig>,
) {
    let handle: Handle<Gltf> = asset_server.load(config.world_assets.zones_path.clone());
    commands.spawn(ZoneLoader {
        handle: handle.clone(),
    });
    info!(
        "Loading world zones from {}",
        config.world_assets.zones_path
    );
}
