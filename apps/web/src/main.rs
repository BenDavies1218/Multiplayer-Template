#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use bevy::prelude::*;
use core::time::Duration;
use lightyear::prelude::*;
use lightyear::prelude::client::InputTimelineConfig;

use game_core::common::cli::{Cli, Mode};
use game_core::common::shared;
use game_core::config;
use game_core::SharedPlugin;
use game_client::{ExampleClientPlugin, FirstPersonPlugin};

fn main() {
    // Load environment variables from .env file (won't exist in WASM, uses defaults)
    config::init();

    let cli = Cli::default();

    let mut app = cli.build_app(Duration::from_secs_f64(1.0 / shared::fixed_timestep_hz()), true);

    app.add_plugins(SharedPlugin);
    app.add_plugins(ExampleClientPlugin);
    app.add_plugins(FirstPersonPlugin::default());

    cli.spawn_connections(&mut app);

    // Set input delay configuration
    if let Some(Mode::Client { .. }) = cli.mode {
        add_input_delay(&mut app);
    }

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
