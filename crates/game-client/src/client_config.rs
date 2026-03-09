use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Client configuration loaded from game_client_config.json
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GameClientConfig {
    pub window: WindowConfig,
    pub input: InputConfig,
    pub rendering: RenderingConfig,
    pub transport: ClientTransportConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InputConfig {
    pub jump_key: String,
    pub jump_gamepad: String,
    pub sprint_key: String,
    pub sprint_gamepad: String,
    pub crouch_key: String,
    pub crouch_gamepad: String,
    pub shoot_key: String,
    pub cursor_grab_button: String,
    pub cursor_release_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RenderingConfig {
    pub camera_start_position: [f32; 3],
    pub eye_height_offset: f32,
    pub projectile_radius: f32,
    pub interpolation_send_ratio: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ClientTransportConfig {
    pub token_expiration: i32,
}

impl Default for GameClientConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig::default(),
            input: InputConfig::default(),
            rendering: RenderingConfig::default(),
            transport: ClientTransportConfig::default(),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Lightyear Example".to_string(),
            width: 1024,
            height: 768,
        }
    }
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            jump_key: "Space".to_string(),
            jump_gamepad: "South".to_string(),
            sprint_key: "ShiftLeft".to_string(),
            sprint_gamepad: "LeftThumb".to_string(),
            crouch_key: "KeyC".to_string(),
            crouch_gamepad: "East".to_string(),
            shoot_key: "KeyQ".to_string(),
            cursor_grab_button: "Left".to_string(),
            cursor_release_key: "Escape".to_string(),
        }
    }
}

impl Default for RenderingConfig {
    fn default() -> Self {
        Self {
            camera_start_position: [0.0, 2.0, 0.0],
            eye_height_offset: 0.5,
            projectile_radius: 1.0,
            interpolation_send_ratio: 2.0,
        }
    }
}

impl Default for ClientTransportConfig {
    fn default() -> Self {
        Self {
            token_expiration: -1,
        }
    }
}

impl GameClientConfig {
    /// Load from JSON file, falling back to defaults if not found
    pub fn load() -> Self {
        let paths = [
            "assets/config/game_client_config.json",
            "../../assets/config/game_client_config.json",
        ];
        for path in &paths {
            if let Ok(contents) = std::fs::read_to_string(path) {
                match serde_json::from_str(&contents) {
                    Ok(config) => return config,
                    Err(e) => {
                        tracing::warn!("Failed to parse {}: {}", path, e);
                    }
                }
            }
        }
        tracing::info!("No game_client_config.json found, using defaults");
        Self::default()
    }
}

/// Parse a gamepad button string to Bevy GamepadButton
pub fn parse_gamepad_button(s: &str) -> Option<GamepadButton> {
    match s {
        "South" => Some(GamepadButton::South),
        "East" => Some(GamepadButton::East),
        "West" => Some(GamepadButton::West),
        "North" => Some(GamepadButton::North),
        "LeftTrigger" => Some(GamepadButton::LeftTrigger),
        "RightTrigger" => Some(GamepadButton::RightTrigger),
        "LeftThumb" => Some(GamepadButton::LeftThumb),
        "RightThumb" => Some(GamepadButton::RightThumb),
        "Start" => Some(GamepadButton::Start),
        "Select" => Some(GamepadButton::Select),
        _ => None,
    }
}

/// Parse a mouse button string
pub fn parse_mouse_button(s: &str) -> Option<MouseButton> {
    match s {
        "Left" => Some(MouseButton::Left),
        "Right" => Some(MouseButton::Right),
        "Middle" => Some(MouseButton::Middle),
        _ => None,
    }
}
