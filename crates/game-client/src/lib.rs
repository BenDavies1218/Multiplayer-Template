//! Game Client - Client-specific game logic and rendering
//!
//! This crate contains all client-side code:
//! - Client app building and transport setup
//! - Client input handling
//! - Rendering and visual effects
//! - Client-side prediction and interpolation

pub mod app;
pub mod character;
pub mod character_rendering;
pub mod client;
pub mod client_config;
pub mod diagnostics;
pub mod dynamic_rendering;
pub mod input_device;
pub mod movement;
pub mod prediction;
pub mod renderer;
pub mod transport;

// Re-export
pub use client::ClientPlugin;
pub use client_config::GameClientConfig;
pub use dynamic_rendering::DynamicRenderingPlugin;
pub use renderer::FirstPersonPlugin;
pub use transport::{ClientTransports, ExampleClient, connect};
