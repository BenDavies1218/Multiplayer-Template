use bevy::prelude::*;
use bevy::gltf::Gltf;
use bevy::scene::{SceneRoot};

use super::{WorldVisual, WorldCollisionLoader, WorldAssets, WorldPluginConfig};

/// Load world assets at startup
///
/// To use this system, place your world files in:
/// - `assets/models/example_world_visual.glb` - High-poly visual scene
/// - `assets/models/example_world_collision.glb` - Low-poly collision scene
///
pub fn load_world_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    config: Res<WorldPluginConfig>,
) {
    let mut visual_handle = None;
    let mut collision_handle = None;

    // Load visual scene if enabled
    if config.load_visual {
        let handle = asset_server.load(format!("{}#Scene0", crate::shared::WORLD_VISUAL_PATH));

        // Spawn the visual scene WITHOUT ColliderConstructor to prevent auto-collider generation
        commands.spawn((
            SceneRoot(handle.clone()),
            WorldVisual,
            Transform::default(),
            GlobalTransform::default(),
        ));

        visual_handle = Some(handle);
        info!("Loading world visual from {}", crate::shared::WORLD_VISUAL_PATH);
    }

    // Load collision mesh if enabled
    if config.load_collision {
        let handle: Handle<Gltf> = asset_server.load(crate::shared::WORLD_COLLISION_PATH);
        commands.spawn(WorldCollisionLoader {
            handle: handle.clone(),
        });

        collision_handle = Some(handle);
        info!("Loading world collision from {}", crate::shared::WORLD_COLLISION_PATH);
    }

    // Store handles in resource for later access
    commands.insert_resource(WorldAssets {
        visual: visual_handle,
        collision: collision_handle,
    });
}