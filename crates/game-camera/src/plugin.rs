use crate::config::{CameraConfig, GameCameraFileConfig};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

/// Main camera plugin for first-person controls
pub struct CameraPlugin {
    pub config: CameraConfig,
}

impl Default for CameraPlugin {
    fn default() -> Self {
        Self {
            config: CameraConfig::default(),
        }
    }
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config.clone());
        app.init_resource::<GameCameraFileConfig>();
        app.add_systems(Update, update_camera_rotation);
    }
}

/// Component for game cameras - stores rotation state
#[derive(Component)]
pub struct GameCamera {
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for GameCamera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

impl GameCamera {
    /// Get the forward direction vector (ignoring pitch, for movement)
    pub fn forward_direction(&self) -> Vec3 {
        let yaw_quat = Quat::from_rotation_y(self.yaw);
        yaw_quat * Vec3::NEG_Z
    }

    /// Get the right direction vector (for strafing)
    pub fn right_direction(&self) -> Vec3 {
        let yaw_quat = Quat::from_rotation_y(self.yaw);
        yaw_quat * Vec3::X
    }
}

/// System to update camera rotation based on mouse input
fn update_camera_rotation(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut camera_query: Query<(&mut GameCamera, &mut Transform)>,
    config: Res<CameraConfig>,
    file_config: Res<GameCameraFileConfig>,
) {
    let Ok((mut game_camera, mut transform)) = camera_query.single_mut() else {
        return;
    };

    let delta = mouse_motion.delta;
    if delta != Vec2::ZERO {
        game_camera.yaw -= delta.x * config.sensitivity;
        game_camera.pitch -= delta.y * config.sensitivity;

        // Clamp pitch to prevent camera flipping
        game_camera.pitch = game_camera.pitch.clamp(
            -file_config.pitch_clamp_radians,
            file_config.pitch_clamp_radians,
        );

        // Apply rotation
        transform.rotation =
            Quat::from_euler(EulerRot::YXZ, game_camera.yaw, game_camera.pitch, 0.0);
    }
}
