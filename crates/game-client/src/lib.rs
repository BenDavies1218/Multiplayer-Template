//! Game Client - Client-specific game logic and rendering
//!
//! This crate contains all client-side code:
//! - Client input handling
//! - Rendering and visual effects
//! - Client-side prediction and interpolation

pub mod client;
pub mod renderer;

// Re-export
pub use client::ExampleClientPlugin;
pub use renderer::ExampleRendererPlugin;
