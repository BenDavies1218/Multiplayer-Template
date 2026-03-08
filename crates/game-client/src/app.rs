//! Client app building utilities

use core::time::Duration;

use bevy::prelude::*;
use bevy::DefaultPlugins;
use bevy::window::PresentMode;
use bevy::winit::WinitSettings;
use lightyear::link::RecvLinkConditioner;
use lightyear::prelude::*;

use game_core::common::cli::log_plugin;
use game_core::common::shared::{CLIENT_PORT, server_addr, SHARED_SETTINGS};

use crate::transport::{ClientTransports, ExampleClient, connect};

pub fn window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: format!("Lightyear Example: {}", env!("CARGO_PKG_NAME")),
            resolution: (1024, 768).into(),
            present_mode: PresentMode::AutoVsync,
            // set to true if we want to capture tab etc in wasm
            prevent_default_event_handling: true,
            ..Default::default()
        }),
        ..default()
    }
}

pub fn new_gui_app(add_inspector: bool) -> App {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .build()
            .set(AssetPlugin {
                // Point to root assets folder from apps/native or apps/web
                file_path: "../../assets".to_string(),
                // https://github.com/bevyengine/bevy/issues/10157
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            })
            .set(log_plugin())
            .set(window_plugin()),
    );
    // we want the same frequency of updates for both focused and unfocused
    // Otherwise when testing the movement can look choppy for unfocused windows
    app.insert_resource(WinitSettings::continuous());

    if add_inspector {
        // Note: bevy_inspector_egui would need to be added as a dependency
        // app.add_plugins(bevy_inspector_egui::bevy_egui::EguiPlugin::default());
        // app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
    }
    app
}

/// Build a client app with GUI and lightyear client plugins
pub fn build_client_app(tick_duration: Duration, add_inspector: bool) -> App {
    let mut app = new_gui_app(add_inspector);
    app.add_plugins(lightyear::prelude::client::ClientPlugins { tick_duration });
    app
}

/// Spawn the client connection entity and add the connect system
pub fn spawn_client_connection(app: &mut App, client_id: u64) {
    let conditioner = LinkConditionerConfig::average_condition();
    app.world_mut()
        .spawn(ExampleClient {
            client_id,
            client_port: CLIENT_PORT,
            server_addr: server_addr(),
            conditioner: Some(RecvLinkConditioner::new(conditioner)),
            transport: ClientTransports::WebTransport,
            shared: SHARED_SETTINGS,
        });
    app.add_systems(Startup, connect);
}
