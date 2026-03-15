//! Gameplay simulation configuration shared between client (prediction) and server (authority).
//!
//! Contains movement physics, character definitions, projectile parameters,
//! and zone behavior settings.

use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Top-level simulation config resource holding all gameplay rule parameters.
///
/// Both the client (for prediction) and server (for authoritative simulation)
/// load these values so that physics and game rules stay in sync.
#[derive(Resource, Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct GameSimulationConfig {
    pub movement: MovementConfig,
    pub character: CharacterConfig,
    pub projectile: ProjectileConfig,
    pub zones: ZonesConfig,
}

impl Default for GameSimulationConfig {
    fn default() -> Self {
        Self {
            movement: MovementConfig::default(),
            character: CharacterConfig::default(),
            projectile: ProjectileConfig::default(),
            zones: ZonesConfig::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct MovementConfig {
    pub max_speed: f32,
    pub max_acceleration: f32,
    /// Deceleration rate when no input is held (key released). Higher = snappier stops.
    pub max_deceleration: f32,
    pub jump_impulse: f32,
    pub sprint_multiplier: f32,
    pub crouch_multiplier: f32,
    pub crouch_capsule_height: f32,
    pub ground_tolerance: f32,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            max_speed: 5.0,
            max_acceleration: 20.0,
            max_deceleration: 40.0,
            jump_impulse: 5.0,
            sprint_multiplier: 1.8,
            crouch_multiplier: 0.4,
            crouch_capsule_height: 0.25,
            ground_tolerance: 0.15,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct CharacterConfig {
    pub capsule_radius: f32,
    pub capsule_height: f32,
    /// Maps model ID (e.g. "default") to the player model GLB path.
    /// The server loads these to extract hitbox node transforms.
    pub model_catalog: HashMap<String, String>,
    /// Defines hitbox regions with damage and collider shape.
    /// Region names must match the `hitbox_region` custom property on model nodes.
    pub hitbox_regions: HashMap<String, HitboxRegionConfig>,
}

impl Default for CharacterConfig {
    fn default() -> Self {
        let mut model_catalog = HashMap::new();
        model_catalog.insert(
            "default".to_string(),
            "models/characters/default/player.glb".to_string(),
        );

        let mut hitbox_regions = HashMap::new();
        hitbox_regions.insert(
            "head".to_string(),
            HitboxRegionConfig {
                damage: 2.0,
                shape: HitboxShape::Capsule {
                    radius: 0.15,
                    half_height: 0.1,
                },
            },
        );
        hitbox_regions.insert(
            "chest".to_string(),
            HitboxRegionConfig {
                damage: 1.0,
                shape: HitboxShape::Box {
                    half_extents: [0.25, 0.3, 0.15],
                },
            },
        );
        hitbox_regions.insert(
            "arm".to_string(),
            HitboxRegionConfig {
                damage: 0.75,
                shape: HitboxShape::Capsule {
                    radius: 0.08,
                    half_height: 0.2,
                },
            },
        );
        hitbox_regions.insert(
            "leg".to_string(),
            HitboxRegionConfig {
                damage: 0.5,
                shape: HitboxShape::Capsule {
                    radius: 0.1,
                    half_height: 0.25,
                },
            },
        );

        Self {
            capsule_radius: 0.5,
            capsule_height: 0.5,
            model_catalog,
            hitbox_regions,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HitboxRegionConfig {
    pub damage: f32,
    pub shape: HitboxShape,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum HitboxShape {
    Capsule { radius: f32, half_height: f32 },
    Box { half_extents: [f32; 3] },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ProjectileConfig {
    pub velocity: f32,
    pub lifetime_ms: u64,
    pub radius: f32,
}

impl Default for ProjectileConfig {
    fn default() -> Self {
        Self {
            velocity: 50.0,
            lifetime_ms: 5000,
            radius: 0.1,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct ZonesConfig {
    pub default_damage: f32,
    pub default_damage_interval: f32,
    pub default_spawn_position: [f32; 3],
}

impl Default for ZonesConfig {
    fn default() -> Self {
        Self {
            default_damage: 10.0,
            default_damage_interval: 1.0,
            default_spawn_position: [0.0, 3.0, 0.0],
        }
    }
}
