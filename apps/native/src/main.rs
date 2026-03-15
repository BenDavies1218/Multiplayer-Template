use game_client::GameClientConfig;
use game_client::app::build_full_client_app;
use game_core::GamePerformanceConfig;
use game_core::simulation_config::GameSimulationConfig;
use game_core::utils::cli::Cli;
use game_core::utils::config_loader::load_config;
use game_core::world_config::GameWorldConfig;
use game_networking::config;

fn main() {
    config::init();

    let simulation_config: GameSimulationConfig = load_config("game_simulation_config.json");
    let performance_config: GamePerformanceConfig = load_config("game_performance_config.json");
    let world_config: GameWorldConfig = load_config("game_world_config.json");
    let client_config: GameClientConfig = load_config("game_client_config.json");

    let cli = Cli::default();
    let client_id = cli
        .client_id()
        .expect("You need to specify a client_id via `-c ID`");

    build_full_client_app(
        simulation_config,
        performance_config,
        world_config,
        client_config,
        client_id,
    )
    .run();
}
