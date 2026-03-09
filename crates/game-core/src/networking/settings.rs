use core::time::Duration;
use super::config::Config;

/// Get send interval from config
pub fn send_interval() -> Duration {
    Config::load().send_interval()
}

#[derive(Copy, Clone, Debug)]
pub struct SharedSettings {
    /// An id to identify the protocol version
    pub protocol_id: u64,

    /// a 32-byte array to authenticate via the Netcode.io protocol
    pub private_key: [u8; 32],
}

// ---------------------------------------------------------------------------
// Config-aware versions
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
