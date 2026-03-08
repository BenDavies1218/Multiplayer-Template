use bevy::prelude::*;
use core::time::Duration;
use lightyear::prelude::*;

use game_core::common::cli::Cli;
use game_core::common::shared;
use game_core::config;
use game_core::SharedPlugin;
use game_core::world::{WorldPlugin, WorldPluginConfig};
use game_client::app::{build_client_app, spawn_client_connection};
use game_client::{ClientPlugin, FirstPersonPlugin};

fn main() {
    // Load environment variables from .env file
    config::init();

    let cli = Cli::default();
    let tick = Duration::from_secs_f64(1.0 / shared::fixed_timestep_hz());

    let mut app = build_client_app(tick, true);

    app.add_plugins(SharedPlugin);
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
