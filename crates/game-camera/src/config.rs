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
}
