//! Game Core - Shared game logic and protocol
//!
//! This crate contains all the shared code between client and server:
//! - Network protocol definitions
//! - Shared game logic
//! - Common utilities and settings

pub mod character;
pub mod core_config;
pub mod movement;
pub mod networking;
pub mod utils;
pub mod world;
pub mod zones;

// Re-export commonly used items
pub use character::{
    CharacterHitboxData, CharacterHitboxMarker, CharacterModelId, HitboxRegion,
    attach_hitbox_to_character,
};
pub use core_config::GameCoreConfig;
pub use networking::config::Config;
pub use networking::protocol::*;
pub use networking::shared::*;
