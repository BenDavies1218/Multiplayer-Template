//! Client skybox sub-plugin — asset loading and camera spawning.
//!
//! Extracted from [`FirstPersonPlugin`](crate::renderer::FirstPersonPlugin) so
//! it can be added independently for debugging or selective composition.

use bevy::{core_pipeline::Skybox, prelude::*};
use game_camera::{GameCamera, GameCameraFileConfig};
use game_core::{skybox::prepare_skybox_cubemap, world::WorldAssets, world_config::GameWorldConfig};

/// Tracks all client assets that must load before the camera spawns.
#[derive(Resource)]
struct PendingClientAssets {
    skybox: Handle<Image>,
    ready: bool,
}

/// Loads the skybox, waits for all client assets, then spawns the camera
/// with the skybox attached.
pub struct ClientSkyboxPlugin;

impl Plugin for ClientSkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, start_loading_assets);
        app.add_systems(Update, check_assets_loaded);
    }
}

/// Kick off loading the skybox. World assets are already loaded by WorldPlugin in Startup.
fn start_loading_assets(
    mut commands: Commands,
    world_config: Res<GameWorldConfig>,
    asset_server: Res<AssetServer>,
) {
    let skybox = asset_server.load(&world_config.world_assets.skybox_path);
    commands.insert_resource(PendingClientAssets {
        skybox,
        ready: false,
    });
}

/// Wait for ALL client assets (world visual, collision, skybox) to finish loading,
/// then spawn the camera with the skybox attached.
fn check_assets_loaded(
    mut commands: Commands,
    mut pending: Option<ResMut<PendingClientAssets>>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    world_assets: Option<Res<WorldAssets>>,
    camera_file_config: Res<GameCameraFileConfig>,
    camera_query: Query<Entity, With<GameCamera>>,
) {
    let Some(ref mut pending) = pending else {
        return;
    };
    if pending.ready {
        return;
    }

    // Check world assets
    if let Some(ref world) = world_assets
        && let Some(ref visual) = world.visual
        && !asset_server.is_loaded_with_dependencies(visual)
    {
        return;
    }

    // Check skybox
    if !asset_server.is_loaded_with_dependencies(&pending.skybox) {
        return;
    }

    // --- All assets loaded ---
    info!("All client assets loaded, spawning camera");
    pending.ready = true;

    // Convert skybox to cubemap
    let skybox_handle = pending.skybox.clone();
    if let Some(image) = images.get_mut(&skybox_handle) {
        prepare_skybox_cubemap(image);
    }

    // Spawn camera if not already present
    if camera_query.is_empty() {
        let pos = camera_file_config.start_position;
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(pos[0], pos[1], pos[2]),
            GameCamera::default(),
            Skybox {
                image: skybox_handle,
                brightness: 100.0,
                rotation: Quat::IDENTITY,
            },
        ));
    }
}
