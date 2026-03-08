//! Game Camera - Camera system with multiple view modes
//!
//! This crate provides a flexible camera system supporting:
//! - Free view (spectator/debug camera)
//! - First person view
//! - Third person view
//!
//! The camera behavior can be configured using `CameraConfig` which provides
//! preset configurations for different use cases.

pub mod config;
pub mod plugin;

pub use config::{CameraConfig, CameraViewMode};
pub use plugin::{CameraPlugin, CameraTarget, GameCamera};
