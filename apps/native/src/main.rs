use bevy::prelude::*;
use core::time::Duration;
use lightyear::prelude::*;

use game_camera::GameCameraFileConfig;
use game_client::app::{build_client_app_from_config, spawn_client_connection_from_config};
use game_client::{ClientPlugin, DynamicRenderingPlugin, FirstPersonPlugin, GameClientConfig};
use game_core::GameCoreConfig;
use game_dynamic::{DynamicPlugin, DynamicPluginConfig};
use game_core::utils::cli::Cli;
use game_core::utils::config_hot_reload::{ConfigHotReloadPlugin, ConfigWatchExt};
use game_core::utils::config_loader::load_config;
use game_core::world::{WorldPlugin, WorldPluginConfig};
use game_core::zones::{ZonePlugin, ZonePluginConfig};
use game_networking::NetworkingPlugin;
use game_networking::config;

fn main() {
    config::init();

    let core_config: GameCoreConfig = load_config("game_core_config.json");
    let client_config: GameClientConfig = load_config("game_client_config.json");
    let camera_config: GameCameraFileConfig = load_config("game_camera_config.json");

    let cli = Cli::default();
    let tick = Duration::from_secs_f64(1.0 / core_config.networking.fixed_timestep_hz);

    let mut app = build_client_app_from_config(tick, &client_config, &core_config);

    app.insert_resource(camera_config.clone());
    app.add_plugins(ConfigHotReloadPlugin::default());
    app.watch_config::<GameCoreConfig>("game_core_config.json");
    app.watch_config::<GameClientConfig>("game_client_config.json");
    app.watch_config::<GameCameraFileConfig>("game_camera_config.json");
    app.add_plugins(NetworkingPlugin {
        config: core_config.clone(),
    });
    app.add_plugins(WorldPlugin {
        config: WorldPluginConfig::client(),
    });
    app.add_plugins(ZonePlugin {
        config: ZonePluginConfig::client(),
    });
    app.add_plugins(DynamicPlugin {
        config: DynamicPluginConfig::client(),
    });
    app.add_plugins(DynamicRenderingPlugin);
    app.add_plugins(ClientPlugin);
    app.add_plugins(FirstPersonPlugin {
        camera_config: game_camera::CameraConfig::first_person_from_config(&camera_config),
    });

    let client_id = cli
        .client_id()
        .expect("You need to specify a client_id via `-c ID`");
    spawn_client_connection_from_config(&mut app, client_id, &core_config, &client_config);

    add_input_delay(&mut app);

    app.run();
}

fn add_input_delay(app: &mut App) {
    use lightyear::prelude::client::{InputDelayConfig, InputTimelineConfig};

    let client = app
        .world_mut()
        .query_filtered::<Entity, With<Client>>()
        .single(app.world_mut())
        .unwrap();

    // Set some input-delay since we are predicting all entities
    app.world_mut().entity_mut(client).insert(
        InputTimelineConfig::default().with_input_delay(InputDelayConfig::fixed_input_delay(0)),
    );
}
