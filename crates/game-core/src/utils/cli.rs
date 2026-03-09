//! Shared CLI utilities for parsing command-line arguments

use bevy::log::{Level, LogPlugin};
use bevy::prelude::default;
use clap::{Parser, Subcommand};

/// CLI options to create an [`App`]
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub mode: Option<Mode>,
}

impl Cli {
    /// Get the client id from the CLI
    pub fn client_id(&self) -> Option<u64> {
        match &self.mode {
            Some(Mode::Client { client_id }) => *client_id,
            Some(Mode::Separate { client_id }) => *client_id,
            Some(Mode::HostClient { client_id }) => *client_id,
            _ => None,
        }
    }
}

#[derive(Subcommand, Debug)]
pub enum Mode {
    /// Runs the app in client mode
    Client {
        #[arg(short, long, default_value = None)]
        client_id: Option<u64>,
    },
    /// Runs the app in server mode
    Server,
    /// Creates two bevy apps: a client app and a server app.
    /// Data gets passed between the two via channels.
    Separate {
        #[arg(short, long, default_value = None)]
        client_id: Option<u64>,
    },
    /// Run the app in host-client mode.
    /// The client and the server will run inside the same app.
    HostClient {
        #[arg(short, long, default_value = None)]
        client_id: Option<u64>,
    },
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Client { client_id: None }
    }
}

impl Default for Cli {
    fn default() -> Self {
        cli()
    }
}

/// Parse the CLI arguments.
/// `clap` doesn't run in wasm, so we simply run in Client mode with a random ClientId
pub fn cli() -> Cli {
    cfg_if::cfg_if! {
        if #[cfg(target_family = "wasm")] {
            let client_id = rand::random::<u64>();
            Cli {
                mode: Some(Mode::Client {
                    client_id: Some(client_id),
                })
            }
        } else {
            Cli::parse()
        }
    }
}

pub fn log_plugin() -> LogPlugin {
    LogPlugin {
        level: Level::INFO,
        filter: "wgpu=error,bevy_render=info,bevy_ecs=warn,bevy_time=warn,naga=warn,bevy_enhanced_input::action::fns=error".to_string(),
        ..default()
    }
}

/// Config-aware log plugin that reads level and filter from `GameCoreConfig`.
pub fn log_plugin_from_config(config: &crate::core_config::GameCoreConfig) -> LogPlugin {
    let level = match config.logging.default_level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => {
            eprintln!(
                "[CONFIG ERROR] Unknown log level '{}', using INFO",
                config.logging.default_level
            );
            Level::INFO
        }
    };
    LogPlugin {
        level,
        filter: config.logging.filter.clone(),
        ..default()
    }
}
