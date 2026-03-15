use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Top-level configuration resource for all game-core systems.
///
/// Every field uses `#[serde(default)]` so that partial JSON files work:
/// any key that is missing simply gets its `Default` value.
#[derive(Resource, Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct GameCoreConfig {
    pub asset_path: String,
    pub networking: NetworkingConfig,
    pub movement: MovementConfig,
    pub character: CharacterConfig,
    pub world_assets: WorldAssetsConfig,
    pub rollback_thresholds: RollbackConfig,
    pub zones: ZonesConfig,
    pub debug_colors: DebugColorsConfig,
    pub debug_toggle_keys: DebugToggleKeysConfig,
    pub logging: LoggingConfig,
    pub projectile: ProjectileConfig,
    pub enable_diagnostics: bool,
}

impl Default for GameCoreConfig {
    fn default() -> Self {
        Self {
            asset_path: "assets".to_string(),
            networking: NetworkingConfig::default(),
            movement: MovementConfig::default(),
            character: CharacterConfig::default(),
            world_assets: WorldAssetsConfig::default(),
            rollback_thresholds: RollbackConfig::default(),
            zones: ZonesConfig::default(),
            debug_colors: DebugColorsConfig::default(),
            debug_toggle_keys: DebugToggleKeysConfig::default(),
            logging: LoggingConfig::default(),
            projectile: ProjectileConfig::default(),
            enable_diagnostics: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Sub-configs
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct NetworkingConfig {
    pub server_host: String,
    pub server_port: u16,
    pub fixed_timestep_hz: f64,
    pub send_interval_hz: f64,
    pub client_timeout_secs: i32,
    pub interpolation_buffer_ms: u64,
    pub client_port: u16,
    pub protocol_id: u64,
    pub steam_app_id: u32,
}

impl Default for NetworkingConfig {
    fn default() -> Self {
        Self {
            server_host: "127.0.0.1".to_string(),
            server_port: 5888,
            fixed_timestep_hz: 64.0,
            send_interval_hz: 64.0,
            client_timeout_secs: 3,
            interpolation_buffer_ms: 100,
            client_port: 0,
            protocol_id: 0,
            steam_app_id: 480,
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
pub struct RollbackConfig {
    /// Base XZ position error (metres) below which rollback is suppressed.
    pub position: f32,
    /// Multiplied by current player speed to extend the threshold at higher speeds.
    /// Represents the prediction lag budget in seconds — e.g. 0.02 allows up to ~1 tick
    /// of prediction offset at full speed before triggering rollback.
    /// At max_speed=5 m/s: threshold = 0.02 + 5×0.02 = 0.12 m
    pub position_speed_factor: f32,
    pub rotation: f32,
    pub linear_velocity: f32,
    pub angular_velocity: f32,
}

impl Default for RollbackConfig {
    fn default() -> Self {
        Self {
            position: 0.02,
            position_speed_factor: 0.02,
            rotation: 0.05,
            linear_velocity: 0.5,
            angular_velocity: 0.5,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct DebugColorsConfig {
    pub collision: [f32; 4],
    pub death_zone: [f32; 4],
    pub damage_zone: [f32; 4],
    pub trigger_zone: [f32; 4],
    pub spawn_point: [f32; 4],
    pub dynamic_object: [f32; 4],
}

impl Default for DebugColorsConfig {
    fn default() -> Self {
        Self {
            collision: [1.0, 0.0, 0.0, 0.3],
            death_zone: [1.0, 0.0, 0.0, 0.3],
            damage_zone: [1.0, 1.0, 0.0, 0.3],
            trigger_zone: [0.0, 0.5, 1.0, 0.3],
            spawn_point: [0.0, 1.0, 0.0, 0.3],
            dynamic_object: [0.0, 1.0, 1.0, 0.3],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct DebugToggleKeysConfig {
    pub collision: String,
    pub zone: String,
    pub dynamic: String,
}

impl Default for DebugToggleKeysConfig {
    fn default() -> Self {
        Self {
            collision: "KeyC".to_string(),
            zone: "KeyZ".to_string(),
            dynamic: "KeyD".to_string(),
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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a string like `"KeyC"`, `"Space"`, `"F1"` etc. to a Bevy [`KeyCode`].
///
/// Returns `None` for unrecognized strings.
pub fn parse_key_code(s: &str) -> Option<KeyCode> {
    // Letter keys: KeyA .. KeyZ
    if s.len() == 4 && s.starts_with("Key") {
        let ch = s.as_bytes()[3];
        if ch.is_ascii_uppercase() {
            return match ch {
                b'A' => Some(KeyCode::KeyA),
                b'B' => Some(KeyCode::KeyB),
                b'C' => Some(KeyCode::KeyC),
                b'D' => Some(KeyCode::KeyD),
                b'E' => Some(KeyCode::KeyE),
                b'F' => Some(KeyCode::KeyF),
                b'G' => Some(KeyCode::KeyG),
                b'H' => Some(KeyCode::KeyH),
                b'I' => Some(KeyCode::KeyI),
                b'J' => Some(KeyCode::KeyJ),
                b'K' => Some(KeyCode::KeyK),
                b'L' => Some(KeyCode::KeyL),
                b'M' => Some(KeyCode::KeyM),
                b'N' => Some(KeyCode::KeyN),
                b'O' => Some(KeyCode::KeyO),
                b'P' => Some(KeyCode::KeyP),
                b'Q' => Some(KeyCode::KeyQ),
                b'R' => Some(KeyCode::KeyR),
                b'S' => Some(KeyCode::KeyS),
                b'T' => Some(KeyCode::KeyT),
                b'U' => Some(KeyCode::KeyU),
                b'V' => Some(KeyCode::KeyV),
                b'W' => Some(KeyCode::KeyW),
                b'X' => Some(KeyCode::KeyX),
                b'Y' => Some(KeyCode::KeyY),
                b'Z' => Some(KeyCode::KeyZ),
                _ => None,
            };
        }
    }

    // Digit keys: Digit0 .. Digit9
    if s.len() == 6 && s.starts_with("Digit") {
        let ch = s.as_bytes()[5];
        if ch.is_ascii_digit() {
            return match ch {
                b'0' => Some(KeyCode::Digit0),
                b'1' => Some(KeyCode::Digit1),
                b'2' => Some(KeyCode::Digit2),
                b'3' => Some(KeyCode::Digit3),
                b'4' => Some(KeyCode::Digit4),
                b'5' => Some(KeyCode::Digit5),
                b'6' => Some(KeyCode::Digit6),
                b'7' => Some(KeyCode::Digit7),
                b'8' => Some(KeyCode::Digit8),
                b'9' => Some(KeyCode::Digit9),
                _ => None,
            };
        }
    }

    // Function keys: F1 .. F12
    if s.starts_with('F')
        && s.len() <= 3
        && let Ok(n) = s[1..].parse::<u8>()
    {
        return match n {
            1 => Some(KeyCode::F1),
            2 => Some(KeyCode::F2),
            3 => Some(KeyCode::F3),
            4 => Some(KeyCode::F4),
            5 => Some(KeyCode::F5),
            6 => Some(KeyCode::F6),
            7 => Some(KeyCode::F7),
            8 => Some(KeyCode::F8),
            9 => Some(KeyCode::F9),
            10 => Some(KeyCode::F10),
            11 => Some(KeyCode::F11),
            12 => Some(KeyCode::F12),
            _ => None,
        };
    }

    // Named keys
    match s {
        "Space" => Some(KeyCode::Space),
        "Escape" => Some(KeyCode::Escape),
        "Enter" => Some(KeyCode::Enter),
        "Tab" => Some(KeyCode::Tab),
        "ShiftLeft" => Some(KeyCode::ShiftLeft),
        "ShiftRight" => Some(KeyCode::ShiftRight),
        "ControlLeft" => Some(KeyCode::ControlLeft),
        "ControlRight" => Some(KeyCode::ControlRight),
        "AltLeft" => Some(KeyCode::AltLeft),
        "AltRight" => Some(KeyCode::AltRight),
        _ => None,
    }
}

/// Build a Bevy [`Color`] from an `[r, g, b, a]` array of floats (0.0 – 1.0).
pub fn color_from_array(rgba: [f32; 4]) -> Color {
    Color::srgba(rgba[0], rgba[1], rgba[2], rgba[3])
}
