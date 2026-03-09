use bevy::prelude::*;
use core::time::Duration;
use lightyear::prelude::*;

use game_core::utils::cli::Cli;
use game_core::utils::config_loader::load_config;
use game_core::networking::config;
use game_core::{GameCoreConfig, SharedPlugin};
use game_core::world::{WorldPlugin, WorldPluginConfig};
use game_client::app::{build_client_app, spawn_client_connection};
use game_client::{ClientPlugin, FirstPersonPlugin};

fn main() {
    // Load environment variables from .env file (won't exist in WASM, uses defaults)
    config::init();

    let core_config: GameCoreConfig = load_config("../../assets", "game_core_config.json");

    let cli = Cli::default();
    let tick = Duration::from_secs_f64(1.0 / core_config.networking.fixed_timestep_hz);

    let mut app = build_client_app(tick, true);

    app.add_plugins(SharedPlugin { config: core_config });
    app.add_plugins(WorldPlugin { config: WorldPluginConfig::client() });
    app.add_plugins(ClientPlugin);
    app.add_plugins(FirstPersonPlugin::default());

    let client_id = cli.client_id().expect("You need to specify a client_id via `-c ID`");
    spawn_client_connection(&mut app, client_id);

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
