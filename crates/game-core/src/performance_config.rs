//! Performance and networking tuning configuration.
//!
//! Contains rollback thresholds, networking tick rates, interpolation settings,
//! and other parameters that affect runtime performance rather than gameplay rules.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Top-level performance config resource for networking, rollback, and runtime tuning.
///
/// These settings control tick rates, interpolation buffers, rollback thresholds,
/// and diagnostic/vsync toggles — everything that affects how smoothly the game
/// runs rather than what the game rules are.
#[derive(Resource, Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct GamePerformanceConfig {
    pub networking: NetworkingPerformanceConfig,
    pub rollback_thresholds: RollbackConfig,
    pub enable_diagnostics: bool,
    pub vsync: bool,
}

impl Default for GamePerformanceConfig {
    fn default() -> Self {
        Self {
            networking: NetworkingPerformanceConfig::default(),
            rollback_thresholds: RollbackConfig::default(),
            enable_diagnostics: false,
            vsync: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct NetworkingPerformanceConfig {
    pub fixed_timestep_hz: f64,
    pub send_interval_hz: f64,
    pub interpolation_buffer_ms: u64,
    pub client_timeout_secs: i32,
    pub protocol_id: u64,
}

impl Default for NetworkingPerformanceConfig {
    fn default() -> Self {
        Self {
            fixed_timestep_hz: 64.0,
            send_interval_hz: 64.0,
            interpolation_buffer_ms: 100,
            client_timeout_secs: 3,
            protocol_id: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct RollbackConfig {
    /// Base XZ position error (metres) below which rollback is suppressed.
    pub position: f32,
    /// Multiplied by current player speed to extend the threshold at higher speeds.
    /// Represents the prediction lag budget in seconds — e.g. 0.02 allows up to ~1 tick
    /// of prediction offset at full speed before triggering rollback.
    /// At max_speed=5 m/s: threshold = 0.02 + 5×0.02 = 0.12 m
    pub position_speed_factor: f32,
    pub rotation: f32,
    pub linear_velocity: f32,
    pub angular_velocity: f32,
}

impl Default for RollbackConfig {
    fn default() -> Self {
        Self {
            position: 0.02,
            position_speed_factor: 0.02,
            rotation: 0.05,
            linear_velocity: 0.5,
            angular_velocity: 0.5,
        }
    }
}
