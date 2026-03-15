use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Debug configs (used as standalone resources by debug systems)
// ---------------------------------------------------------------------------

#[derive(Resource, Serialize, Deserialize, Clone, Debug)]
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

#[derive(Resource, Serialize, Deserialize, Clone, Debug)]
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

/// Build a Bevy [`Color`] from an `[r, g, b, a]` array of floats (0.0 -- 1.0).
pub fn color_from_array(rgba: [f32; 4]) -> Color {
    Color::srgba(rgba[0], rgba[1], rgba[2], rgba[3])
}
