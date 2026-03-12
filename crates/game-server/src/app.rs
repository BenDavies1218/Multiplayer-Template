//! Server app building utilities

use core::time::Duration;

use bevy::diagnostic::DiagnosticsPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use game_core::GameCoreConfig;
use game_core::networking::settings::shared_settings_from_config;
use game_core::utils::cli::log_plugin_from_config;

use crate::server_config::GameServerConfig;
use crate::transport::{ExampleServer, ServerTransports, WebTransportCertificateSettings, start};

pub fn new_headless_app_from_config(core_config: &GameCoreConfig) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin {
            file_path: game_core::utils::config_loader::resolve_asset_path_for_bevy(),
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

/// Build a server app with headless mode and lightyear server plugins using config
pub fn build_server_app_from_config(tick_duration: Duration, core_config: &GameCoreConfig) -> App {
    let mut app = new_headless_app_from_config(core_config);
    app.add_plugins(lightyear::prelude::server::ServerPlugins { tick_duration });
    app
}

/// Spawn the server connection entity using config values.
/// Transport type is determined by game_server_config.json (defaults to UDP).
pub fn spawn_server_connection_from_config(app: &mut App, core_config: &GameCoreConfig) {
    let server_config = app
        .world()
        .get_resource::<GameServerConfig>()
        .cloned()
        .unwrap_or_default();
    let shared = shared_settings_from_config(core_config);
    let port = core_config.networking.server_port;

    let transport = match server_config.transport.transport_type.as_str() {
        "webtransport" => ServerTransports::WebTransport {
            local_port: port,
            certificate: WebTransportCertificateSettings::from_config(&server_config.transport),
        },
        "websocket" => ServerTransports::WebSocket { local_port: port },
        _ => ServerTransports::Udp { local_port: port },
    };

    app.world_mut().spawn(ExampleServer {
        conditioner: None,
        transport,
        shared,
    });
    app.add_systems(Startup, start);
}
