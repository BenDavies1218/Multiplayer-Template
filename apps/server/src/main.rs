use core::time::Duration;

use game_core::performance_config::GamePerformanceConfig;
use game_core::simulation_config::GameSimulationConfig;
use game_core::utils::config_hot_reload::{ConfigHotReloadPlugin, ConfigWatchExt};
use game_core::utils::config_loader::load_config;
use game_core::world::WorldPluginConfig;
use game_core::world_config::GameWorldConfig;
use game_core::zones::ZonePluginConfig;
use game_dynamic::{DynamicPlugin, DynamicPluginConfig};
use game_networking::NetworkingPlugin;
use game_networking::config;
use game_server::app::{build_server_app_from_config, spawn_server_connection_from_config};
use game_server::{GameServerConfig, ServerPlugin};

fn main() {
    config::init();

    let simulation_config: GameSimulationConfig = load_config("game_simulation_config.json");
    let performance_config: GamePerformanceConfig = load_config("game_performance_config.json");
    let world_config: GameWorldConfig = load_config("game_world_config.json");
    let server_config: GameServerConfig = load_config("game_server_config.json");

    let tick = Duration::from_secs_f64(1.0 / performance_config.networking.fixed_timestep_hz);

    let mut app = build_server_app_from_config(tick, &world_config);

    let diag_interval = server_config.diagnostics_log_interval_secs;
    app.insert_resource(server_config.clone());
    app.add_plugins(ConfigHotReloadPlugin::default());
    app.watch_config::<GameSimulationConfig>("game_simulation_config.json");
    app.watch_config::<GamePerformanceConfig>("game_performance_config.json");
    app.watch_config::<GameWorldConfig>("game_world_config.json");
    app.watch_config::<GameServerConfig>("game_server_config.json");
    app.add_plugins(NetworkingPlugin {
        simulation: simulation_config,
        performance: performance_config.clone(),
    });
    app.add_plugins(game_core::world::WorldPlugin {
        config: WorldPluginConfig::server(),
    });
    app.add_plugins(game_core::zones::ZonePlugin {
        config: ZonePluginConfig::server(),
    });
    app.add_plugins(DynamicPlugin {
        config: DynamicPluginConfig::server(),
    });
    app.add_plugins(game_core::character::CharacterPlugin);
    app.add_plugins(ServerPlugin);

    if performance_config.enable_diagnostics {
        app.insert_resource(game_diagnostics::TargetTickRate(
            performance_config.networking.fixed_timestep_hz,
        ));
        app.add_plugins(game_diagnostics::DiagnosticsPlugin::server_with_interval(
            diag_interval,
        ));
    }

    spawn_server_connection_from_config(&mut app, &server_config, &performance_config);

    app.run();
}
