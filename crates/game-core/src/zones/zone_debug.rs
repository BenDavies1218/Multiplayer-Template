use crate::core_config::{GameCoreConfig, color_from_array, parse_key_code};
use bevy::prelude::*;

/// Resource to control zone debug visualization
#[derive(Resource, Debug)]
pub struct ZoneDebugSettings {
    /// Whether zone meshes should be visible
    pub visible: bool,
    /// Color for death zones
    pub death_zone_color: Color,
    /// Color for damage zones
    pub damage_zone_color: Color,
    /// Color for trigger zones
    pub trigger_zone_color: Color,
    /// Color for spawn points
    pub spawn_point_color: Color,
}

impl Default for ZoneDebugSettings {
    fn default() -> Self {
        Self {
            visible: false,
            death_zone_color: Color::srgba(1.0, 0.0, 0.0, 0.3),
            damage_zone_color: Color::srgba(1.0, 1.0, 0.0, 0.3),
            trigger_zone_color: Color::srgba(0.0, 0.5, 1.0, 0.3),
            spawn_point_color: Color::srgba(0.0, 1.0, 0.0, 0.3),
        }
    }
}

/// Marker component for zone debug visualization meshes
#[derive(Component, Debug)]
pub struct ZoneDebugMesh;

/// System to toggle zone debug visualization with configurable key
pub fn toggle_zone_debug(
    keys: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<ZoneDebugSettings>,
    config: Res<GameCoreConfig>,
) {
    let key = parse_key_code(&config.debug_toggle_keys.zone).unwrap_or(KeyCode::KeyZ);
    if keys.just_pressed(key) {
        settings.visible = !settings.visible;
        info!(
            "Zone debug visualization: {}",
            if settings.visible { "ON" } else { "OFF" }
        );
    }
}

/// System to update visibility of zone debug meshes
pub fn update_zone_debug_visibility(
    settings: Res<ZoneDebugSettings>,
    mut debug_meshes: Query<&mut Visibility, With<ZoneDebugMesh>>,
) {
    if settings.is_changed() {
        let visibility = if settings.visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        for mut mesh_visibility in debug_meshes.iter_mut() {
            *mesh_visibility = visibility;
        }
    }
}

/// Startup system to apply zone debug colors from `GameCoreConfig`.
pub fn apply_zone_debug_config(
    config: Res<GameCoreConfig>,
    mut settings: ResMut<ZoneDebugSettings>,
) {
    settings.death_zone_color = color_from_array(config.debug_colors.death_zone);
    settings.damage_zone_color = color_from_array(config.debug_colors.damage_zone);
    settings.trigger_zone_color = color_from_array(config.debug_colors.trigger_zone);
    settings.spawn_point_color = color_from_array(config.debug_colors.spawn_point);
}
