use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Server configuration loaded from game_server_config.json
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GameServerConfig {
    pub spawning: SpawningConfig,
    pub transport: ServerTransportJsonConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SpawningConfig {
    pub fallback_angle_multiplier: f32,
    pub fallback_radius: f32,
    pub fallback_height: f32,
    pub player_colors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerTransportJsonConfig {
    /// Transport type: "udp", "webtransport", or "websocket"
    pub transport_type: String,
    pub certificate_sans: Vec<String>,
}

impl Default for SpawningConfig {
    fn default() -> Self {
        Self {
            fallback_angle_multiplier: 5.0,
            fallback_radius: 2.0,
            fallback_height: 3.0,
            player_colors: vec![
                "limegreen".to_string(),
                "pink".to_string(),
                "yellow".to_string(),
                "aqua".to_string(),
                "crimson".to_string(),
                "gold".to_string(),
                "orange_red".to_string(),
                "silver".to_string(),
                "salmon".to_string(),
                "yellow_green".to_string(),
                "white".to_string(),
                "red".to_string(),
            ],
        }
    }
}

impl Default for ServerTransportJsonConfig {
    fn default() -> Self {
        Self {
            transport_type: "udp".to_string(),
            certificate_sans: vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "::1".to_string(),
            ],
        }
    }
}

/// Parse a CSS color name string to a Bevy Color.
/// Falls back to white if unrecognized.
pub fn parse_css_color(name: &str) -> Color {
    use bevy::color::palettes::css;
    match name {
        "limegreen" => Color::from(css::LIMEGREEN),
        "pink" => Color::from(css::PINK),
        "yellow" => Color::from(css::YELLOW),
        "aqua" => Color::from(css::AQUA),
        "crimson" => Color::from(css::CRIMSON),
        "gold" => Color::from(css::GOLD),
        "orange_red" => Color::from(css::ORANGE_RED),
        "silver" => Color::from(css::SILVER),
        "salmon" => Color::from(css::SALMON),
        "yellow_green" => Color::from(css::YELLOW_GREEN),
        "white" => Color::from(css::WHITE),
        "red" => Color::from(css::RED),
        "blue" => Color::from(css::BLUE),
        "green" => Color::from(css::GREEN),
        "orange" => Color::from(css::ORANGE),
        "purple" => Color::from(css::PURPLE),
        "magenta" => Color::from(css::MAGENTA),
        "cyan" => Color::from(css::AQUA), // CSS cyan = aqua
        _ => {
            warn!("[config] Unknown color '{}', using white", name);
            Color::from(css::WHITE)
        }
    }
}
