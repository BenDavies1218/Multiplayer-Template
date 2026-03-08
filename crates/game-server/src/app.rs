//! Server app building utilities

use core::time::Duration;

use bevy::prelude::*;
use bevy::diagnostic::DiagnosticsPlugin;
use bevy::state::app::StatesPlugin;

use game_core::common::cli::log_plugin;
use game_core::common::shared::{server_port, SHARED_SETTINGS};

use crate::transport::{ExampleServer, ServerTransports, WebTransportCertificateSettings, start};

pub fn new_headless_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin {
            file_path: "../../assets".to_string(),
            meta_check: bevy::asset::AssetMetaCheck::Never,
            ..default()
        },
        log_plugin(),
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

/// Build a server app with headless mode and lightyear server plugins
pub fn build_server_app(tick_duration: Duration) -> App {
    let mut app = new_headless_app();
    app.add_plugins(lightyear::prelude::server::ServerPlugins { tick_duration });
    app
}

/// Spawn the server connection entity and add the start system
pub fn spawn_server_connection(app: &mut App) {
    app.world_mut()
        .spawn(ExampleServer {
            conditioner: None,
            transport: ServerTransports::WebTransport {
                local_port: server_port(),
                certificate: WebTransportCertificateSettings::FromFile {
                    cert: "./certificates/cert.pem".to_string(),
                    key: "./certificates/key.pem".to_string(),
                },
            },
            shared: SHARED_SETTINGS,
        });
    app.add_systems(Startup, start);
}
