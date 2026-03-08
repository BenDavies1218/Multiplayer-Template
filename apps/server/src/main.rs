use core::time::Duration;

use game_core::common::shared;
use game_core::config;
use game_core::SharedPlugin;
use game_core::world::{WorldPlugin, WorldPluginConfig};
use game_server::app::{build_server_app, spawn_server_connection};
use game_server::ServerPlugin;

fn main() {
    config::init();

    let tick = Duration::from_secs_f64(1.0 / shared::fixed_timestep_hz());

    let mut app = build_server_app(tick);

    app.add_plugins(SharedPlugin);
    app.add_plugins(WorldPlugin { config: WorldPluginConfig::server() });
    app.add_plugins(ServerPlugin);

    spawn_server_connection(&mut app);

    app.run();
}
