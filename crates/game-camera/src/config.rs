use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Camera view mode determines how the camera behaves and what it follows
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CameraViewMode {
    /// Free-flying camera for spectating/debugging (noclip)
    FreeView,
    /// First-person camera attached to player entity
    FirstPerson,
    /// Third-person camera following player entity
    ThirdPerson,
}

/// Configuration for camera behavior
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct CameraConfig {
    /// Current camera view mode
    pub view_mode: CameraViewMode,
    /// Mouse sensitivity for camera rotation
    pub sensitivity: f32,
    /// Movement speed for free camera
    pub free_camera_speed: f32,
    /// Distance from player in third-person mode
    pub third_person_distance: f32,
    /// Height offset from player in third-person mode
    pub third_person_height: f32,
    /// Enable camera smoothing/lerping
    pub smooth_camera: bool,
    /// Camera smoothing factor (0.0 = no smoothing, 1.0 = instant)
    pub smooth_factor: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self::first_person()
    }
}

impl CameraConfig {
    /// Server configuration: no camera rendering
    pub fn server() -> Self {
        Self {
            view_mode: CameraViewMode::FirstPerson,
            sensitivity: 0.0,
            free_camera_speed: 0.0,
            third_person_distance: 0.0,
            third_person_height: 0.0,
            smooth_camera: false,
            smooth_factor: 0.0,
        }
    }

    /// Client configuration: first-person view with standard settings
    pub fn client() -> Self {
        Self::first_person()
    }

    /// Viewer configuration: free camera for world inspection
    pub fn viewer() -> Self {
        Self::free_view()
    }

    /// First-person view configuration
    pub fn first_person() -> Self {
        Self {
            view_mode: CameraViewMode::FirstPerson,
            sensitivity: 0.002,
            free_camera_speed: 0.0,
            third_person_distance: 0.0,
            third_person_height: 0.0,
            smooth_camera: false,
            smooth_factor: 0.1,
        }
    }

    /// Third-person view configuration
    pub fn third_person() -> Self {
        Self {
            view_mode: CameraViewMode::ThirdPerson,
            sensitivity: 0.002,
            free_camera_speed: 0.0,
            third_person_distance: 5.0,
            third_person_height: 2.0,
            smooth_camera: true,
            smooth_factor: 0.1,
        }
    }

    /// Free view (spectator) configuration
    pub fn free_view() -> Self {
        Self {
            view_mode: CameraViewMode::FreeView,
            sensitivity: 0.002,
            free_camera_speed: 10.0,
            third_person_distance: 0.0,
            third_person_height: 0.0,
            smooth_camera: false,
            smooth_factor: 0.1,
        }
    }

    /// First-person view from file config
    pub fn first_person_from_config(file_config: &GameCameraFileConfig) -> Self {
        let p = &file_config.first_person;
        Self {
            view_mode: CameraViewMode::FirstPerson,
            sensitivity: p.sensitivity,
            free_camera_speed: p.free_camera_speed,
            third_person_distance: p.third_person_distance,
            third_person_height: p.third_person_height,
            smooth_camera: p.smooth_camera,
            smooth_factor: p.smooth_factor,
        }
    }

    /// Third-person view from file config
    pub fn third_person_from_config(file_config: &GameCameraFileConfig) -> Self {
        let p = &file_config.third_person;
        Self {
            view_mode: CameraViewMode::ThirdPerson,
            sensitivity: p.sensitivity,
            free_camera_speed: p.free_camera_speed,
            third_person_distance: p.third_person_distance,
            third_person_height: p.third_person_height,
            smooth_camera: p.smooth_camera,
            smooth_factor: p.smooth_factor,
        }
    }

    /// Free view from file config
    pub fn free_view_from_config(file_config: &GameCameraFileConfig) -> Self {
        let p = &file_config.free_view;
        Self {
            view_mode: CameraViewMode::FreeView,
            sensitivity: p.sensitivity,
            free_camera_speed: p.free_camera_speed,
            third_person_distance: p.third_person_distance,
            third_person_height: p.third_person_height,
            smooth_camera: p.smooth_camera,
            smooth_factor: p.smooth_factor,
        }
    }
}

/// Configuration for a single camera preset, loaded from file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CameraPresetConfig {
    pub sensitivity: f32,
    pub free_camera_speed: f32,
    pub third_person_distance: f32,
    pub third_person_height: f32,
    pub smooth_camera: bool,
    pub smooth_factor: f32,
}

impl Default for CameraPresetConfig {
    fn default() -> Self {
        Self {
            sensitivity: 0.002,
            free_camera_speed: 0.0,
            third_person_distance: 0.0,
            third_person_height: 0.0,
            smooth_camera: false,
            smooth_factor: 0.1,
        }
    }
}

/// File-level camera configuration loaded from game_camera_config.json
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GameCameraFileConfig {
    pub first_person: CameraPresetConfig,
    pub third_person: CameraPresetConfig,
    pub free_view: CameraPresetConfig,
    pub pitch_clamp_radians: f32,
}

impl Default for GameCameraFileConfig {
    fn default() -> Self {
        Self {
            first_person: CameraPresetConfig {
                sensitivity: 0.002,
                free_camera_speed: 0.0,
                third_person_distance: 0.0,
                third_person_height: 0.0,
                smooth_camera: false,
                smooth_factor: 0.1,
            },
            third_person: CameraPresetConfig {
                sensitivity: 0.002,
                free_camera_speed: 0.0,
                third_person_distance: 5.0,
                third_person_height: 2.0,
                smooth_camera: true,
                smooth_factor: 0.1,
            },
            free_view: CameraPresetConfig {
                sensitivity: 0.002,
                free_camera_speed: 10.0,
                third_person_distance: 0.0,
                third_person_height: 0.0,
                smooth_camera: false,
                smooth_factor: 0.1,
            },
            pitch_clamp_radians: 1.54,
        }
    }
}
