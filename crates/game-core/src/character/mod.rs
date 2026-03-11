use bevy::prelude::*;

use crate::core_config::GameCoreConfig;

pub mod components;
pub mod hitbox_loader;

pub use components::{CharacterHitboxMarker, CharacterModelId, HitboxRegion};
pub use hitbox_loader::{
    CharacterHitboxData, CharacterHitboxLoader, HitboxRegionData, attach_hitbox_to_character,
    process_character_hitbox,
};

/// Plugin for character model and hitbox systems.
///
/// Handles:
/// - Loading the character hitbox GLB and parsing regions with damage attributes
/// - Providing `CharacterHitboxData` resource for attaching hitboxes on spawn
/// - Registering `CharacterModelId` for replication
pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        // Load hitbox GLB on startup
        app.add_systems(Startup, load_character_hitbox);
        // Process hitbox once asset is ready
        app.add_systems(Update, process_character_hitbox);
    }
}

fn load_character_hitbox(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<GameCoreConfig>,
) {
    for (id, path) in &config.character.hitbox_catalog {
        let handle = asset_server.load(path.clone());
        commands.spawn(CharacterHitboxLoader { handle });
        info!("Loading character hitbox '{}' from {}", id, path);
    }
}
