//! Game Core - Shared game logic and configuration
//!
//! This crate contains non-networking shared code:
//! - Configuration loading and types
//! - Character models and hitboxes
//! - World asset loading and processing
//! - Zone detection and events
//! - Common utilities

pub mod character;
pub mod core_config;
pub mod skybox;
pub mod utils;
pub mod world;
pub mod zones;

// Re-export commonly used items
pub use character::{
    CharacterHitboxData, CharacterHitboxMarker, CharacterMarker, CharacterModelId, HitboxRegion,
    attach_hitbox_to_character,
};
pub use core_config::{GameCoreConfig, HitboxRegionConfig, HitboxShape};
