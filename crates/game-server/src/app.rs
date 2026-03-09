//! Server app building utilities

use core::time::Duration;

use bevy::prelude::*;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::state::app::StatesPlugin;

use game_core::utils::cli::log_plugin_from_config;
use game_core::networking::settings::shared_settings_from_config;
use game_core::GameCoreConfig;

use crate::transport::{ExampleServer, ServerTransports, start};

pub fn new_headless_app() -> App {
    new_headless_app_from_config(&GameCoreConfig::default())
}

pub fn new_headless_app_from_config(core_config: &GameCoreConfig) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin {
            file_path: "../../assets".to_string(),
            meta_check: bevy::asset::AssetMetaCheck::Never,
            ..default()
        },
        log_plugin_from_config(core_config),
        StatesPlugin,
        DiagnosticsPlugin,
    ));

    // Add minimal plugins for loading collision meshes from GLTF files
    // Server needs these to load collision geometry from .glb files
    app.add_plugins(bevy::gltf::GltfPlugin::default());
    app.add_plugins(bevy::transform::TransformPlugin);
    app.add_plugins(bevy::scene::ScenePlugin);
    // Initialize all asset types that GLTF files might reference
    app.init_asset::<bevy::pbr::StandardMaterial>();
    app.init_asset::<bevy::mesh::Mesh>();
    app.init_asset::<bevy::scene::Scene>();
    app.init_asset::<Image>();

    app
}

/// Build a server app with headless mode and lightyear server plugins (uses defaults)
pub fn build_server_app(tick_duration: Duration) -> App {
    build_server_app_from_config(tick_duration, &GameCoreConfig::default())
}

/// Build a server app with headless mode and lightyear server plugins using config
pub fn build_server_app_from_config(tick_duration: Duration, core_config: &GameCoreConfig) -> App {
    let mut app = new_headless_app_from_config(core_config);
    app.add_plugins(lightyear::prelude::server::ServerPlugins { tick_duration });
    app
}

/// Spawn the server connection entity and add the start system (uses defaults)
pub fn spawn_server_connection(app: &mut App) {
    let core_config = GameCoreConfig::default();
    spawn_server_connection_from_config(app, &core_config);
}

/// Spawn the server connection entity using config values
pub fn spawn_server_connection_from_config(app: &mut App, core_config: &GameCoreConfig) {
    app.world_mut()
        .spawn(ExampleServer {
            conditioner: None,
            transport: ServerTransports::Udp {
                local_port: core_config.networking.server_port,
            },
            shared: shared_settings_from_config(core_config),
        });
    app.add_systems(Startup, start);
}
