use crate::core_config::{GameCoreConfig, parse_key_code};
use bevy::prelude::*;

/// Resource to control collision mesh visualization
#[derive(Resource, Debug)]
pub struct CollisionDebugSettings {
    /// Whether collision meshes should be visible
    pub visible: bool,
    /// Color to use for collision mesh visualization
    pub color: Color,
}

impl Default for CollisionDebugSettings {
    fn default() -> Self {
        Self {
            visible: false,
            color: Color::srgba(1.0, 0.0, 0.0, 0.3),
        }
    }
}

/// Marker component for collision debug visualization meshes
#[derive(Component, Debug)]
pub struct CollisionDebugMesh;

/// System to toggle collision mesh visualization with a keyboard shortcut
pub fn toggle_collision_debug(
    keys: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<CollisionDebugSettings>,
    config: Res<GameCoreConfig>,
) {
    let key = parse_key_code(&config.debug_toggle_keys.collision).unwrap_or(KeyCode::KeyC);
    if keys.just_pressed(key) {
        settings.visible = !settings.visible;
        info!(
            "Collision debug visualization: {}",
            if settings.visible { "ON" } else { "OFF" }
        );
    }
}

/// System to update visibility of collision debug meshes
pub fn update_collision_debug_visibility(
    settings: Res<CollisionDebugSettings>,
    mut debug_meshes: Query<&mut Visibility, With<CollisionDebugMesh>>,
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

/// Startup system to apply collision debug colors from `GameCoreConfig`.
pub fn apply_debug_config(
    config: Res<GameCoreConfig>,
    mut settings: ResMut<CollisionDebugSettings>,
) {
    settings.color = crate::core_config::color_from_array(config.debug_colors.collision);
}
