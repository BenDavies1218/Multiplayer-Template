use bevy::prelude::*;
use core::time::Duration;
use lightyear::prelude::*;

use game_core::utils::cli::Cli;
use game_core::utils::config_loader::load_config;
use game_core::networking::config;
use game_core::{GameCoreConfig, SharedPlugin};
use game_core::world::{WorldPlugin, WorldPluginConfig};
use game_client::app::{build_client_app_from_config, spawn_client_connection_from_config};
use game_client::{ClientPlugin, FirstPersonPlugin, GameClientConfig};
use game_camera::GameCameraFileConfig;

fn main() {
    // Load environment variables from .env file (won't exist in WASM, uses defaults)
    config::init();

    let core_config: GameCoreConfig = load_config("game_core_config.json");
    let client_config: GameClientConfig = load_config("game_client_config.json");
    let camera_config: GameCameraFileConfig = load_config("game_camera_config.json");

    unsafe { std::env::set_var("BEVY_ASSET_ROOT", &core_config.asset_path); }

    let cli = Cli::default();
    let tick = Duration::from_secs_f64(1.0 / core_config.networking.fixed_timestep_hz);

    let mut app = build_client_app_from_config(tick, true, &client_config, &core_config);

    app.insert_resource(camera_config.clone());
    app.add_plugins(SharedPlugin { config: core_config.clone() });
    app.add_plugins(WorldPlugin { config: WorldPluginConfig::client() });
    app.add_plugins(ClientPlugin);
    app.add_plugins(FirstPersonPlugin {
        camera_config: game_camera::CameraConfig::first_person_from_config(&camera_config),
    });

    let client_id = cli.client_id().expect("You need to specify a client_id via `-c ID`");
    spawn_client_connection_from_config(&mut app, client_id, &core_config);

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
