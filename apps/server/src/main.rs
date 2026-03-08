#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use bevy::prelude::*;
use core::time::Duration;
use lightyear::prelude::*;

use game_core::common::cli::{Cli, Mode};
use game_core::common::shared;
use game_core::config;
use game_core::SharedPlugin;
use game_server::ServerPlugin;

fn main() {
    config::init();

    let cli = Cli::default();

    let mut app = cli.build_app(Duration::from_secs_f64(1.0 / shared::fixed_timestep_hz()), false);

    app.add_plugins(SharedPlugin);
    app.add_plugins(ServerPlugin);

    cli.spawn_connections(&mut app);

    app.run();
}
