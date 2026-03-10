//! Client app building utilities

use core::time::Duration;

use bevy::prelude::*;
use bevy::DefaultPlugins;
use bevy::window::PresentMode;
use bevy::winit::WinitSettings;
use lightyear::link::RecvLinkConditioner;
use lightyear::prelude::*;

use game_core::utils::cli::log_plugin_from_config;
use game_core::networking::settings::{shared_settings_from_config};
use game_core::GameCoreConfig;

use crate::client_config::GameClientConfig;
use crate::transport::{ClientTransports, ExampleClient, connect};

pub fn window_plugin() -> WindowPlugin {
    window_plugin_from_config(&GameClientConfig::default())
}

pub fn window_plugin_from_config(config: &GameClientConfig) -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: format!("{}: {}", config.window.title, env!("CARGO_PKG_NAME")),
            resolution: (config.window.width, config.window.height).into(),
            present_mode: PresentMode::AutoVsync,
            // set to true if we want to capture tab etc in wasm
            prevent_default_event_handling: true,
            ..Default::default()
        }),
        ..default()
    }
}

pub fn new_gui_app(add_inspector: bool) -> App {
    new_gui_app_from_config(add_inspector, &GameClientConfig::default(), &GameCoreConfig::default())
}

pub fn new_gui_app_from_config(add_inspector: bool, config: &GameClientConfig, core_config: &GameCoreConfig) -> App {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .build()
            .set(AssetPlugin {
                file_path: game_core::utils::config_loader::resolve_asset_path_for_bevy(),
                // https://github.com/bevyengine/bevy/issues/10157
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            })
            .set(log_plugin_from_config(core_config))
            .set(window_plugin_from_config(config)),
    );
    // we want the same frequency of updates for both focused and unfocused
    // Otherwise when testing the movement can look choppy for unfocused windows
    app.insert_resource(WinitSettings::continuous());
    app.insert_resource(config.clone());

    if add_inspector {
        // Note: bevy_inspector_egui would need to be added as a dependency
        // app.add_plugins(bevy_inspector_egui::bevy_egui::EguiPlugin::default());
        // app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
    }
    app
}

/// Build a client app with GUI and lightyear client plugins (uses defaults)
pub fn build_client_app(tick_duration: Duration, add_inspector: bool) -> App {
    let mut app = new_gui_app(add_inspector);
    app.add_plugins(lightyear::prelude::client::ClientPlugins { tick_duration });
    app
}

/// Build a client app using config
pub fn build_client_app_from_config(tick_duration: Duration, add_inspector: bool, config: &GameClientConfig, core_config: &GameCoreConfig) -> App {
    let mut app = new_gui_app_from_config(add_inspector, config, core_config);
    app.add_plugins(lightyear::prelude::client::ClientPlugins { tick_duration });
    app
}

/// Spawn the client connection entity and add the connect system (uses defaults)
pub fn spawn_client_connection(app: &mut App, client_id: u64) {
    let core_config = GameCoreConfig::default();
    spawn_client_connection_from_config(app, client_id, &core_config);
}

/// Spawn the client connection entity using config values
pub fn spawn_client_connection_from_config(app: &mut App, client_id: u64, core_config: &GameCoreConfig) {
    let conditioner = LinkConditionerConfig::average_condition();
    app.world_mut()
        .spawn(ExampleClient {
            client_id,
            client_port: core_config.networking.client_port,
            server_addr: game_core::networking::config::Config::from_core_config(core_config).server_addr(),
            conditioner: Some(RecvLinkConditioner::new(conditioner)),
            transport: {
                #[cfg(not(target_family = "wasm"))]
                { ClientTransports::Udp }
                #[cfg(target_family = "wasm")]
                { ClientTransports::WebTransport }
            },
            shared: shared_settings_from_config(core_config),
        });
    app.add_systems(Startup, connect);
}
