use core::net::{SocketAddr};
use core::time::Duration;
use super::config::Config;

/// Get fixed timestep frequency from config
pub fn fixed_timestep_hz() -> f64 {
    Config::load().fixed_timestep_hz
}

/// Get server port from config
pub fn server_port() -> u16 {
    Config::load().server_port
}

/// 0 means that the OS will assign any available port
pub const CLIENT_PORT: u16 = 0;

/// Get server address from config
pub fn server_addr() -> SocketAddr {
    Config::load().server_addr()
}

/// Get send interval from config
pub fn send_interval() -> Duration {
    Config::load().send_interval()
}

pub const SHARED_SETTINGS: SharedSettings = SharedSettings {
    protocol_id: 0,
    private_key: [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ],
};

pub const STEAM_APP_ID: u32 = 480; // Steamworks App ID for Spacewar, used for testing

#[derive(Copy, Clone, Debug)]
pub struct SharedSettings {
    /// An id to identify the protocol version
    pub protocol_id: u64,

    /// a 32-byte array to authenticate via the Netcode.io protocol
    pub private_key: [u8; 32],
}

// ---------------------------------------------------------------------------
// Config-aware versions (prefer these over the const/function versions above)
// ---------------------------------------------------------------------------

use crate::core_config::GameCoreConfig;

pub fn client_port_from_config(config: &GameCoreConfig) -> u16 {
    config.networking.client_port
}

pub fn shared_settings_from_config(config: &GameCoreConfig) -> SharedSettings {
    SharedSettings {
        protocol_id: config.networking.protocol_id,
        private_key: [0u8; 32],
    }
}

pub fn steam_app_id_from_config(config: &GameCoreConfig) -> u32 {
    config.networking.steam_app_id
}
