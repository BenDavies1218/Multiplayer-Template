use core::time::Duration;

use game_core::utils::config_loader::load_config;
use game_core::networking::config;
use game_core::{GameCoreConfig, SharedPlugin};
use game_core::world::{WorldPlugin, WorldPluginConfig};
use game_core::zones::{ZonePlugin, ZonePluginConfig};
use game_server::app::{build_server_app_from_config, spawn_server_connection_from_config};
use game_server::{GameServerConfig, ServerPlugin};

fn main() {
    config::init();

    let core_config: GameCoreConfig = load_config("game_core_config.json");
    let server_config: GameServerConfig = load_config("game_server_config.json");

    let tick = Duration::from_secs_f64(1.0 / core_config.networking.fixed_timestep_hz);

    let mut app = build_server_app_from_config(tick, &core_config);

    app.insert_resource(server_config);
    app.add_plugins(SharedPlugin { config: core_config.clone() });
    app.add_plugins(WorldPlugin { config: WorldPluginConfig::server() });
    app.add_plugins(ZonePlugin { config: ZonePluginConfig::server() });
    app.add_plugins(game_core::player::PlayerPlugin);
    app.add_plugins(ServerPlugin);

    spawn_server_connection_from_config(&mut app, &core_config);

    app.run();
}
