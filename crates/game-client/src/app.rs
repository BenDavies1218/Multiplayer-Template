//! Client app building utilities

use core::time::Duration;

use bevy::DefaultPlugins;
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::winit::WinitSettings;
use lightyear::link::RecvLinkConditioner;
use lightyear::prelude::*;

use game_core::performance_config::GamePerformanceConfig;
use game_core::simulation_config::GameSimulationConfig;
use game_core::utils::cli::log_plugin_from_config;
use game_core::utils::config_hot_reload::{ConfigHotReloadPlugin, ConfigWatchExt};
use game_core::world_config::GameWorldConfig;
use game_networking::config::shared_settings_from_performance;

use crate::client_config::GameClientConfig;
use crate::transport::{ClientTransports, ExampleClient, connect};

pub fn window_plugin_from_config(
    config: &GameClientConfig,
    performance_config: &GamePerformanceConfig,
) -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: format!("{}: {}", config.window.title, env!("CARGO_PKG_NAME")),
            resolution: (config.window.width, config.window.height).into(),
            present_mode: if performance_config.vsync {
                PresentMode::AutoVsync
            } else {
                PresentMode::AutoNoVsync
            },
            // set to true if we want to capture tab etc in wasm
            prevent_default_event_handling: true,
            ..Default::default()
        }),
        ..default()
    }
}

pub fn new_gui_app_from_config(
    config: &GameClientConfig,
    world_config: &GameWorldConfig,
    performance_config: &GamePerformanceConfig,
) -> App {
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
            .set(log_plugin_from_config(world_config))
            .set(window_plugin_from_config(config, performance_config)),
    );
    // we want the same frequency of updates for both focused and unfocused
    // Otherwise when testing the movement can look choppy for unfocused windows
    app.insert_resource(WinitSettings::continuous());
    app.insert_resource(config.clone());

    app
}

/// Build a client app using config
pub fn build_client_app_from_config(
    tick_duration: Duration,
    config: &GameClientConfig,
    world_config: &GameWorldConfig,
    performance_config: &GamePerformanceConfig,
) -> App {
    let mut app = new_gui_app_from_config(config, world_config, performance_config);
    app.add_plugins(lightyear::prelude::client::ClientPlugins { tick_duration });
    app
}

/// Spawn the client connection entity using config values
pub fn spawn_client_connection_from_config(
    app: &mut App,
    client_id: u64,
    client_config: &GameClientConfig,
    performance_config: &GamePerformanceConfig,
    world_config: &GameWorldConfig,
) {
    let conditioner = if client_config.transport.simulate_latency {
        Some(RecvLinkConditioner::new(
            LinkConditionerConfig::average_condition(),
        ))
    } else {
        None
    };
    app.world_mut().spawn(ExampleClient {
        client_id,
        client_port: client_config.connection.client_port,
        server_addr: game_networking::config::Config::from_configs(
            performance_config,
            world_config,
            &client_config.connection.server_host,
            client_config.connection.server_port,
        )
        .server_addr(),
        conditioner,
        transport: {
            #[cfg(not(target_family = "wasm"))]
            {
                ClientTransports::Udp
            }
            #[cfg(target_family = "wasm")]
            {
                ClientTransports::WebTransport
            }
        },
        shared: shared_settings_from_performance(performance_config),
    });
    app.add_systems(Startup, connect);
}

/// Build a complete client app with all plugins.
///
/// This is the standard client setup used by both native and web apps.
/// It adds config hot-reload, networking, world/zone/dynamic plugins (client mode),
/// rendering, input, prediction, and camera — then sets up the transport connection
/// and input delay.
pub fn build_full_client_app(
    simulation_config: GameSimulationConfig,
    performance_config: GamePerformanceConfig,
    world_config: GameWorldConfig,
    client_config: GameClientConfig,
    client_id: u64,
) -> App {
    let tick = Duration::from_secs_f64(1.0 / performance_config.networking.fixed_timestep_hz);
    let mut app =
        build_client_app_from_config(tick, &client_config, &world_config, &performance_config);

    // Insert camera file config as resource (used by ClientSkyboxPlugin)
    app.insert_resource(client_config.camera.clone());

    app.add_plugins(ConfigHotReloadPlugin::default());
    app.watch_config::<GameSimulationConfig>("game_simulation_config.json");
    app.watch_config::<GamePerformanceConfig>("game_performance_config.json");
    app.watch_config::<GameWorldConfig>("game_world_config.json");
    app.watch_config::<GameClientConfig>("game_client_config.json");
    app.add_plugins(game_networking::NetworkingPlugin {
        simulation: simulation_config.clone(),
        performance: performance_config.clone(),
    });
    app.add_plugins(game_core::world::WorldPlugin {
        config: game_core::world::WorldPluginConfig::client(),
    });
    app.add_plugins(game_core::zones::ZonePlugin {
        config: game_core::zones::ZonePluginConfig::client(),
    });
    app.add_plugins(game_dynamic::DynamicPlugin {
        config: game_dynamic::DynamicPluginConfig::client(),
    });
    app.add_plugins(crate::DynamicRenderingPlugin);
    app.add_plugins(crate::ClientPlugin);
    app.add_plugins(crate::FirstPersonPlugin {
        camera_config: game_camera::CameraConfig::first_person_from_config(&client_config.camera),
    });

    // Insert debug config resources as standalone resources for game-core debug systems
    app.insert_resource(client_config.debug.colors.clone());
    app.insert_resource(client_config.debug.toggle_keys.clone());

    if performance_config.enable_diagnostics {
        app.add_plugins(game_diagnostics::DiagnosticsPlugin::client());
    }

    spawn_client_connection_from_config(
        &mut app,
        client_id,
        &client_config,
        &performance_config,
        &world_config,
    );

    // Input delay configuration
    {
        use lightyear::prelude::client::{InputDelayConfig, InputTimelineConfig};
        let client_entity = app
            .world_mut()
            .query_filtered::<Entity, With<Client>>()
            .single(app.world_mut())
            .unwrap();
        app.world_mut().entity_mut(client_entity).insert(
            InputTimelineConfig::default().with_input_delay(InputDelayConfig::fixed_input_delay(0)),
        );
    }

    app
}
