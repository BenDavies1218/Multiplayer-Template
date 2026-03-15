use std::collections::HashMap;

use bevy::prelude::*;
use game_core::core_config::{DebugColorsConfig, DebugToggleKeysConfig};
use leafwing_input_manager::prelude::GamepadStick;
use serde::{Deserialize, Serialize};

/// Connection settings for the client (server address, ports).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ClientConnectionConfig {
    pub server_host: String,
    pub server_port: u16,
    pub client_port: u16,
}

impl Default for ClientConnectionConfig {
    fn default() -> Self {
        Self {
            server_host: "127.0.0.1".to_string(),
            server_port: 5888,
            client_port: 0,
        }
    }
}

/// Debug visualization and toggle key settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DebugConfig {
    pub colors: DebugColorsConfig,
    pub toggle_keys: DebugToggleKeysConfig,
}

/// Client configuration loaded from game_client_config.json
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct GameClientConfig {
    pub connection: ClientConnectionConfig,
    pub window: WindowConfig,
    pub input: InputConfig,
    pub rendering: RenderingConfig,
    pub camera: game_camera::GameCameraFileConfig,
    pub character: CharacterClientConfig,
    pub transport: ClientTransportConfig,
    pub debug: DebugConfig,
    pub enable_diagnostics: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum InputDevice {
    #[default]
    Auto,
    KeyboardMouse,
    Gamepad,
}

/// The actual device currently in use — never `Auto`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource)]
pub enum ResolvedDevice {
    #[default]
    KeyboardMouse,
    Gamepad,
}

/// Runtime resource tracking which input device is actually active.
#[derive(Debug, Resource)]
pub struct ActiveInputDevice {
    /// The resolved device in use right now.
    pub device: ResolvedDevice,
    /// The first connected gamepad entity, if any.
    pub gamepad: Option<Entity>,
}

impl Default for ActiveInputDevice {
    fn default() -> Self {
        Self {
            device: ResolvedDevice::KeyboardMouse,
            gamepad: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InputConfig {
    pub active_device: InputDevice,
    pub keyboard: KeyboardBindings,
    pub gamepad: GamepadBindings,
    pub cursor_grab_button: String,
    pub cursor_release_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyboardBindings {
    // Movement
    pub move_up: String,
    pub move_down: String,
    pub move_left: String,
    pub move_right: String,
    pub sprint: String,
    pub crouch: String,
    pub prone: String,
    pub jump: String,
    pub mount_ledge: String,
    // Combat
    pub fire: String,
    pub aim_down_sights: String,
    pub reload: String,
    pub primary_weapon: String,
    pub secondary_weapon: String,
    pub interact: String,
    pub lethal_equipment: String,
    pub tactical_equipment: String,
    pub melee: String,
    pub weapon_inspect: String,
    pub armor_plate: String,
    pub alternate_fire: String,
    // Killstreaks
    pub killstreak1: String,
    pub killstreak2: String,
    pub killstreak3: String,
    pub field_upgrade: String,
    // Communication
    pub text_chat: String,
    pub team_chat: String,
    pub ping: String,
    pub push_to_talk: String,
    pub gesture1: String,
    pub gesture2: String,
    pub gesture3: String,
    pub gesture4: String,
    // Misc
    pub scoreboard: String,
    pub map: String,
    pub inventory: String,
    pub pause: String,
    pub night_vision: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GamepadBindings {
    // Movement
    pub move_stick: String,
    pub look_stick: String,
    pub sprint: String,
    pub crouch: String,
    pub jump: String,
    // Combat
    pub fire: String,
    pub aim_down_sights: String,
    pub reload: String,
    pub switch_weapon: String,
    pub lethal_equipment: String,
    pub tactical_equipment: String,
    pub melee: String,
    pub killstreak1: String,
    pub killstreak2: String,
    pub killstreak3: String,
    pub field_upgrade: String,
    // Other
    pub ping: String,
    pub armor_plate: String,
    pub night_vision: String,
    pub scoreboard: String,
    pub pause: String,
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
    /// Simulate network latency (~100ms jitter) using Lightyear's link conditioner.
    /// Useful for testing latency compensation locally. Disable for accurate local play.
    pub simulate_latency: bool,
}

/// Describes the set of models for a single character type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CharacterModelSet {
    /// Third-person model path (what other clients see)
    pub player: String,
    /// First-person POV model — empty hands
    pub pov_empty: String,
    /// First-person POV weapon models, keyed by weapon name
    #[serde(default)]
    pub pov_weapons: HashMap<String, String>,
}

impl Default for CharacterModelSet {
    fn default() -> Self {
        Self {
            player: "models/characters/default/player.glb".to_string(),
            pov_empty: "models/characters/default/pov_empty.glb".to_string(),
            pov_weapons: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CharacterClientConfig {
    pub model_catalog: HashMap<String, CharacterModelSet>,
    pub selected_model: String,
}

impl Default for CharacterClientConfig {
    fn default() -> Self {
        let mut catalog = HashMap::new();
        catalog.insert("default".to_string(), CharacterModelSet::default());
        Self {
            model_catalog: catalog,
            selected_model: "default".to_string(),
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
            active_device: InputDevice::Auto,
            keyboard: KeyboardBindings::default(),
            gamepad: GamepadBindings::default(),
            cursor_grab_button: "Left".to_string(),
            cursor_release_key: "Escape".to_string(),
        }
    }
}

impl Default for KeyboardBindings {
    fn default() -> Self {
        Self {
            move_up: "KeyW".to_string(),
            move_down: "KeyS".to_string(),
            move_left: "KeyA".to_string(),
            move_right: "KeyD".to_string(),
            sprint: "ShiftLeft".to_string(),
            crouch: "ControlLeft".to_string(),
            prone: "KeyC".to_string(),
            jump: "Space".to_string(),
            mount_ledge: "Space".to_string(),
            fire: "".to_string(),
            aim_down_sights: "".to_string(),
            reload: "KeyR".to_string(),
            primary_weapon: "Digit1".to_string(),
            secondary_weapon: "Digit2".to_string(),
            interact: "KeyE".to_string(),
            lethal_equipment: "KeyQ".to_string(),
            tactical_equipment: "".to_string(),
            melee: "KeyV".to_string(),
            weapon_inspect: "KeyG".to_string(),
            armor_plate: "KeyF".to_string(),
            alternate_fire: "KeyB".to_string(),
            killstreak1: "Digit3".to_string(),
            killstreak2: "Digit4".to_string(),
            killstreak3: "Digit5".to_string(),
            field_upgrade: "KeyX".to_string(),
            text_chat: "KeyT".to_string(),
            team_chat: "KeyY".to_string(),
            ping: "KeyZ".to_string(),
            push_to_talk: "CapsLock".to_string(),
            gesture1: "F1".to_string(),
            gesture2: "F2".to_string(),
            gesture3: "F3".to_string(),
            gesture4: "F4".to_string(),
            scoreboard: "Tab".to_string(),
            map: "KeyM".to_string(),
            inventory: "KeyI".to_string(),
            pause: "Escape".to_string(),
            night_vision: "KeyN".to_string(),
        }
    }
}

impl Default for GamepadBindings {
    fn default() -> Self {
        Self {
            move_stick: "LeftStick".to_string(),
            look_stick: "RightStick".to_string(),
            sprint: "LeftThumb".to_string(),
            crouch: "RightThumb".to_string(),
            jump: "South".to_string(),
            fire: "RightTrigger".to_string(),
            aim_down_sights: "LeftTrigger".to_string(),
            reload: "West".to_string(),
            switch_weapon: "North".to_string(),
            lethal_equipment: "LeftTrigger2".to_string(),
            tactical_equipment: "RightTrigger2".to_string(),
            melee: "East".to_string(),
            killstreak1: "DPadRight".to_string(),
            killstreak2: "DPadRight".to_string(),
            killstreak3: "DPadRight".to_string(),
            field_upgrade: "DPadLeft".to_string(),
            ping: "DPadLeft".to_string(),
            armor_plate: "DPadUp".to_string(),
            night_vision: "DPadDown".to_string(),
            scoreboard: "Select".to_string(),
            pause: "Start".to_string(),
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
            simulate_latency: false,
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
        "LeftTrigger2" => Some(GamepadButton::LeftTrigger2),
        "RightTrigger2" => Some(GamepadButton::RightTrigger2),
        "DPadUp" => Some(GamepadButton::DPadUp),
        "DPadDown" => Some(GamepadButton::DPadDown),
        "DPadLeft" => Some(GamepadButton::DPadLeft),
        "DPadRight" => Some(GamepadButton::DPadRight),
        _ => None,
    }
}

/// Parse a gamepad stick string to a leafwing GamepadStick.
pub fn parse_gamepad_stick(s: &str) -> Option<GamepadStick> {
    match s {
        "LeftStick" => Some(GamepadStick::LEFT),
        "RightStick" => Some(GamepadStick::RIGHT),
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
