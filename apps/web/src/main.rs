use game_camera::GameCameraFileConfig;
use game_client::GameClientConfig;
use game_client::app::build_full_client_app;
use game_core::GameCoreConfig;
use game_core::utils::cli::Cli;
use game_core::utils::config_loader::load_config;
use game_networking::config;

fn main() {
    // Load environment variables from .env file (won't exist in WASM, uses defaults)
    config::init();

    let core_config: GameCoreConfig = load_config("game_core_config.json");
    let client_config: GameClientConfig = load_config("game_client_config.json");
    let camera_config: GameCameraFileConfig = load_config("game_camera_config.json");

    let cli = Cli::default();
    let client_id = cli
        .client_id()
        .expect("You need to specify a client_id via `-c ID`");

    build_full_client_app(core_config, client_config, camera_config, client_id).run();
}
