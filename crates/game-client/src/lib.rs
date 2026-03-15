//! Game Client — client-specific game logic and rendering.
//!
//! # Sub-plugin architecture
//!
//! The two main convenience plugins, [`ClientPlugin`] and [`FirstPersonPlugin`],
//! are thin wrappers that compose smaller, independently-usable sub-plugins:
//!
//! ## [`ClientPlugin`] wraps:
//! - [`InputPlugin`] — input device detection, InputMap rebuilding, camera-to-ActionState sync
//! - [`PredictionPlugin`] — predicted movement simulation and diagnostics
//! - [`LifecyclePlugin`] — attaches InputMap/physics/orientation to new predicted characters
//!
//! ## [`FirstPersonPlugin`] wraps:
//! - [`CameraPlugin`](game_camera::CameraPlugin) — camera modes from `game-camera`
//! - [`CharacterRenderingPlugin`] — character model preloading and attachment
//! - [`ClientSkyboxPlugin`] — skybox loading and camera spawning
//! - [`VisualInterpolationPlugin`] — frame interpolation for smooth rendering
//! - [`ProjectileCosmeticsPlugin`] — projectile sphere meshes and colliders
//! - [`CursorPlugin`] — cursor grab/release handling
//! - [`AnimationPlugin`] — auto-play glTF animations
//!
//! Apps can replace a convenience plugin with a hand-picked subset of
//! sub-plugins for debugging or special-purpose builds.

pub mod animation_plugin;
pub mod app;
pub mod character;
pub mod character_rendering;
pub mod client;
pub mod client_config;
pub mod client_skybox_plugin;
pub mod cursor_plugin;
pub mod diagnostics;
pub mod dynamic_rendering;
pub mod input_device;
pub mod input_plugin;
pub mod lifecycle_plugin;
pub mod movement;
pub mod prediction;
pub mod prediction_plugin;
pub mod projectile_cosmetics_plugin;
pub mod renderer;
pub mod transport;
pub mod visual_interpolation_plugin;

// Re-export convenience plugins
pub use client::ClientPlugin;
pub use renderer::FirstPersonPlugin;

// Re-export sub-plugins
pub use animation_plugin::AnimationPlugin;
pub use client_skybox_plugin::ClientSkyboxPlugin;
pub use cursor_plugin::CursorPlugin;
pub use input_plugin::InputPlugin;
pub use lifecycle_plugin::LifecyclePlugin;
pub use prediction_plugin::PredictionPlugin;
pub use projectile_cosmetics_plugin::ProjectileCosmeticsPlugin;
pub use visual_interpolation_plugin::VisualInterpolationPlugin;

// Re-export other public types
pub use client_config::{
    ClientConnectionConfig, DebugConfig, GameClientConfig,
};
pub use dynamic_rendering::DynamicRenderingPlugin;
pub use transport::{ClientTransports, ExampleClient, connect};
