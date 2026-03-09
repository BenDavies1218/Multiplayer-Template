//! Game Core - Shared game logic and protocol
//!
//! This crate contains all the shared code between client and server:
//! - Network protocol definitions
//! - Shared game logic
//! - Common utilities and settings

pub mod core_config;
pub mod networking;
pub mod utils;
pub mod world;
pub mod movement;
pub mod zones;

// Re-export commonly used items
pub use networking::protocol::*;
pub use networking::shared::*;
pub use networking::config::Config;
pub use core_config::GameCoreConfig;
