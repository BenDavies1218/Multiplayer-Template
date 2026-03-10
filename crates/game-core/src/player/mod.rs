use bevy::prelude::*;

use crate::core_config::GameCoreConfig;

pub mod components;
pub mod hitbox_loader;

pub use components::{PlayerModelId, PlayerHitboxMarker, HitboxRegion};
pub use hitbox_loader::{
    PlayerHitboxLoader, PlayerHitboxData, HitboxRegionData,
    process_player_hitbox, attach_hitbox_to_player,
};

/// Plugin for player model and hitbox systems.
///
/// Handles:
/// - Loading the player hitbox GLB and parsing regions with damage attributes
/// - Providing `PlayerHitboxData` resource for attaching hitboxes on spawn
/// - Registering `PlayerModelId` for replication
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // Load hitbox GLB on startup
        app.add_systems(Startup, load_player_hitbox);
        // Process hitbox once asset is ready
        app.add_systems(Update, process_player_hitbox);
    }
}

fn load_player_hitbox(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<GameCoreConfig>,
) {
    let handle = asset_server.load(config.world_assets.player_hitbox_path.clone());
    commands.spawn(PlayerHitboxLoader { handle });
    info!("Loading player hitbox from {}", config.world_assets.player_hitbox_path);
}
