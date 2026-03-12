use bevy::prelude::*;

use crate::core_config::GameCoreConfig;

pub mod components;
pub mod hitbox_loader;

pub use components::{CharacterHitboxMarker, CharacterMarker, CharacterModelId, HitboxRegion};
pub use hitbox_loader::{
    CharacterHitboxData, CharacterModelLoader, HitboxRegionData, attach_hitbox_to_character,
    process_character_model_hitboxes,
};

/// Plugin for character model and hitbox systems.
///
/// Handles:
/// - Loading the player model GLB and scanning for hitbox region nodes
/// - Creating simple shape colliders from config, positioned at tagged node transforms
/// - Providing `CharacterHitboxData` resource for attaching hitboxes on spawn
pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        // Load player model GLB(s) on startup to extract hitbox node transforms
        app.add_systems(Startup, load_character_models);
        // Process model once asset is ready — extract hitbox regions
        app.add_systems(Update, process_character_model_hitboxes);
    }
}

fn load_character_models(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<GameCoreConfig>,
) {
    for (id, path) in &config.character.model_catalog {
        let handle = asset_server.load(path.clone());
        commands.spawn(CharacterModelLoader { handle });
        info!(
            "Loading character model '{}' from {} for hitbox extraction",
            id, path
        );
    }
}
