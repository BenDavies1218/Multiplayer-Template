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
pub mod performance_config;
pub mod simulation_config;
pub mod skybox;
pub mod utils;
pub mod world;
pub mod world_config;
pub mod zones;

// Re-export commonly used items
pub use character::{
    CharacterHitboxData, CharacterHitboxMarker, CharacterMarker, CharacterModelId, HitboxRegion,
    attach_hitbox_to_character,
};
pub use core_config::{DebugColorsConfig, DebugToggleKeysConfig};
pub use performance_config::GamePerformanceConfig;
pub use simulation_config::GameSimulationConfig;
pub use world_config::GameWorldConfig;
