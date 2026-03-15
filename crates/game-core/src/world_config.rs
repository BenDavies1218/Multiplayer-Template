//! World and environment configuration.
//!
//! Contains world asset paths, logging settings, and the top-level
//! asset directory configuration.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Top-level world config resource for asset paths and logging.
///
/// Controls where world assets are loaded from and how logging is configured.
#[derive(Resource, Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct GameWorldConfig {
    pub asset_path: String,
    pub world_assets: WorldAssetsConfig,
    pub logging: LoggingConfig,
}

impl Default for GameWorldConfig {
    fn default() -> Self {
        Self {
            asset_path: "assets".to_string(),
            world_assets: WorldAssetsConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct WorldAssetsConfig {
    pub visual_path: String,
    pub dynamic_path: String,
    pub zones_path: String,
    pub skybox_path: String,
}

impl Default for WorldAssetsConfig {
    fn default() -> Self {
        Self {
            visual_path: "models/example_world_visual.glb".to_string(),
            dynamic_path: "models/world_dynamic.glb".to_string(),
            zones_path: "models/example_world_zones.glb".to_string(),
            skybox_path: "sunset_sky_hdr.exr".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct LoggingConfig {
    pub default_level: String,
    pub filter: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            default_level: "info".to_string(),
            filter: "wgpu=error,bevy_render=info,bevy_ecs=warn,bevy_time=warn,naga=warn,bevy_enhanced_input::action::fns=error".to_string(),
        }
    }
}
