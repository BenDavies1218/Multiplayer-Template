use game_core::core_config::{GameCoreConfig, color_from_array, parse_key_code};
use bevy::prelude::*;

/// Resource to control dynamic object debug visualization.
#[derive(Resource, Debug)]
pub struct DynamicDebugSettings {
    pub visible: bool,
    pub dynamic_object_color: Color,
}

impl Default for DynamicDebugSettings {
    fn default() -> Self {
        Self {
            visible: false,
            dynamic_object_color: Color::srgba(0.0, 1.0, 1.0, 0.3),
        }
    }
}

/// Marker component for dynamic object debug visualization meshes.
#[derive(Component, Debug)]
pub struct DynamicDebugMesh;

/// Startup system to apply debug colors from config.
pub fn apply_dynamic_debug_config(
    config: Res<GameCoreConfig>,
    mut settings: ResMut<DynamicDebugSettings>,
) {
    settings.dynamic_object_color = color_from_array(config.debug_colors.dynamic_object);
}

/// Toggle dynamic debug visualization with configurable key.
pub fn toggle_dynamic_debug(
    keys: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<DynamicDebugSettings>,
    config: Res<GameCoreConfig>,
) {
    let key = parse_key_code(&config.debug_toggle_keys.dynamic).unwrap_or(KeyCode::KeyD);
    if keys.just_pressed(key) {
        settings.visible = !settings.visible;
        info!(
            "Dynamic debug visualization: {}",
            if settings.visible { "ON" } else { "OFF" }
        );
    }
}

/// Update visibility of dynamic debug meshes.
pub fn update_dynamic_debug_visibility(
    settings: Res<DynamicDebugSettings>,
    mut debug_meshes: Query<&mut Visibility, With<DynamicDebugMesh>>,
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
